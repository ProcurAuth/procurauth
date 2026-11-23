//! The PEP in a plain HTTP service, with no MCP. Each test is one item of the section
//! "Use from a plain HTTP service" of the README. The tests also run without the feature `mcp`.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use std::sync::Arc;
use std::time::Duration;

use common::*;
use procurauth_core::path::NodePath;
use procurauth_pep::operation::{Operation, node};
use procurauth_pep::refusal::APPROVAL_CANNOT_WAIT;
use procurauth_pep::{Identity, Pep, Refusal};
use serde_json::{Value, json};

const URL: &str = "http://pdp.test";

fn table(database: &str, name: &str) -> NodePath {
    node(
        SERVICE,
        &[("host", "prod"), ("database", database), ("table", name)],
    )
    .unwrap()
}

fn database(name: &str) -> NodePath {
    node(SERVICE, &[("host", "prod"), ("database", name)]).unwrap()
}

fn service(
    transport: Arc<Scripted>,
    tweak: impl FnOnce(&mut procurauth_pep::config::PepConfig),
) -> (Pep, Identity) {
    let edge = Edge::new();
    let (pep, _) = pep(transport, &edge, URL, tweak);
    // The request of a client: the service gives the headers as name and value pairs.
    let token = edge.token("alice");
    let headers = [("Accept", "application/json"), (HEADER, token.as_str())];
    let who = pep.authenticate(headers).unwrap();
    (pep, who)
}

fn body(reply: &procurauth_pep::reply::HttpReply) -> Value {
    serde_json::from_slice(&reply.body).unwrap()
}

#[test]
fn authenticate_reads_the_token_from_the_configured_header_only() {
    let edge = Edge::new();
    let (pep, _) = pep(yes_to_all(), &edge, URL, |_| {});
    assert_eq!(pep.token_header(), HEADER);
    let token = edge.token("alice");
    let who = pep.authenticate([(HEADER, token.as_str())]).unwrap();
    assert_eq!(who.sub(), "alice");
    // A token in `Authorization` is not an identity token: the reply is 401.
    let bearer = format!("Bearer {token}");
    let refusal = pep
        .authenticate([("Authorization", bearer.as_str())])
        .unwrap_err();
    let reply = refusal.to_http();
    assert_eq!(reply.status, 401);
    assert_eq!(body(&reply), json!({"error": "unauthorized"}));
}

#[test]
fn an_operation_asks_its_other_permissions_with_permission_only_first() {
    let t = yes_to_all();
    let (pep, who) = service(t.clone(), |_| {});
    // The copy of 10.1a: write `test2`, and read the table `orders` of `test1`.
    let copy =
        Operation::new("data.write", database("test2")).also("data.read", table("test1", "orders"));
    let allowed = pep.decide(&who, &copy).into_result().unwrap();
    let posts = t.posts();
    let other = posts[0].body.as_ref().unwrap();
    assert_eq!(other["action"]["name"], "data.read");
    assert_eq!(other["context"]["permission_only"], true);
    assert!(other.pointer("/context/usage").is_none());
    let used = posts[1].body.as_ref().unwrap();
    assert!(used.pointer("/context/permission_only").is_none());
    assert_eq!(allowed.others().len(), 1);
}

#[test]
fn decide_sends_the_usage_for_an_action_that_lists_a_budget() {
    let t = yes_to_all();
    let (pep, who) = service(t.clone(), |_| {});
    let allowed = pep
        .decide(&who, &Operation::new("data.read", table("test2", "orders")))
        .into_result()
        .unwrap();
    assert_eq!(allowed.units(), 1);
    let posts = t.posts();
    assert_eq!(posts.len(), 1);
    assert_eq!(
        posts[0].body.as_ref().unwrap()["context"]["usage"],
        json!({"units": 1})
    );
}

#[test]
fn a_route_that_cannot_wait_for_an_approval_answers_403_approval_cannot_wait() {
    let t = Scripted::answering(
        |_| json!({"decision": false, "context": {"reason": "approval_required", "evaluation_id": "ev_1"}}),
    );
    let (pep, who) = service(t.clone(), |_| {});
    let operation = Operation::new("data.write", database("test2"));
    // One call gives the refusal, and one call gives the reply.
    let reply = pep
        .decide(&who, &operation)
        .into_result()
        .unwrap_err()
        .to_http();
    assert_eq!(reply.status, 403);
    assert_eq!(reply.header("content-type"), Some("application/json"));
    assert_eq!(body(&reply), json!({"error": APPROVAL_CANNOT_WAIT}));
    assert_eq!(APPROVAL_CANNOT_WAIT, "approval_cannot_wait");
    // `decide` sends no effect and submits no request.
    assert!(t.posts().iter().all(|p| {
        p.body
            .as_ref()
            .unwrap()
            .pointer("/context/approval")
            .is_none()
    }));
    assert!(
        t.posts()
            .iter()
            .all(|p| p.path() == "/access/v1/evaluation")
    );
}

#[test]
fn the_pep_keeps_no_answer_with_a_reason_of_17_1_and_no_true_answer_with_units() {
    let t = Scripted::answering(|q| {
        if q["action"]["name"] == "data.write" {
            json!({"decision": false, "context": {"reason": "approval_pending", "evaluation_id": "ev_1"}})
        } else {
            let usage = q.pointer("/context/usage").is_some();
            yes(json!({}), usage.then_some(1))
        }
    });
    let (pep, who) = service(t.clone(), |c| {
        c.cache_ttl = Duration::from_secs(60);
    });
    for _ in 0..2 {
        let write = pep.decide(&who, &Operation::new("data.write", database("test2")));
        assert_eq!(write.into_result().unwrap_err().status(), 403);
        let read = pep.decide(&who, &Operation::new("data.read", table("test2", "orders")));
        assert_eq!(read.into_result().unwrap().units(), 1);
    }
    // Each call asked the PDP again: no answer was kept.
    assert_eq!(t.posts().len(), 4);
    assert_eq!(pep.kept_answers(), 0);
}

#[test]
fn the_metadata_gives_the_published_limits_and_the_usage_wait_is_longer() {
    let t = Scripted::new(|seen| {
        Ok(if seen.method == "GET" {
            json_response(
                200,
                &json!({
                    "policy_decision_point": URL,
                    "procurauth_usage_max_seconds": 8,
                    "procurauth_idle_max_seconds": 30,
                    "procurauth_evaluations_max_bytes": 65536,
                    "procurauth_evaluations_max_items": 20,
                    "procurauth_directory_max_age_seconds": 120
                }),
            )
        } else {
            json_response(200, &yes(json!({}), seen.has_usage().then_some(1)))
        })
    });
    let (pep, who) = service(t.clone(), |c| {
        c.pdp.request_timeout = Duration::from_secs(3);
        c.pdp.usage_wait_margin = Duration::from_secs(2);
    });
    let meta = pep.metadata().unwrap();
    assert_eq!(meta.usage_max_seconds, Some(8));
    assert_eq!(meta.idle_max_seconds, Some(30));
    assert_eq!(meta.evaluations_max_bytes, Some(65_536));
    assert_eq!(meta.evaluations_max_items, Some(20));
    assert_eq!(meta.directory_max_age_seconds(), Some(120));
    pep.decide(&who, &Operation::new("data.read", table("test2", "orders")))
        .into_result()
        .unwrap();
    // 10.6: the wait for a question with usage is the published time plus the margin.
    let posts = t.posts();
    assert_eq!(posts[0].timeout, Duration::from_secs(10));
}

#[test]
fn the_token_cache_avoids_a_second_check_of_the_same_token() {
    let edge = Edge::new();
    let (pep, clock) = pep(yes_to_all(), &edge, URL, |c| {
        c.token_cache_max_entries = 100;
    });
    let token = edge.token("alice");
    for _ in 0..3 {
        pep.authenticate([(HEADER, token.as_str())]).unwrap();
    }
    assert_eq!(pep.kept_tokens(), 1);
    // The token of the tests has `exp` NOW + 290, and the skew is 30 seconds.
    clock.set_seconds(NOW + 290 + 30 + 1);
    assert!(matches!(
        pep.authenticate([(HEADER, token.as_str())]),
        Err(Refusal::Unauthenticated(_))
    ));
    assert_eq!(pep.kept_tokens(), 0);
}

#[test]
fn p_10_1c_a_route_with_no_action_asks_access_on_the_service() {
    let t = yes_to_all();
    let (pep, who) = service(t.clone(), |_| {});
    pep.decide_route_without_action(&who).unwrap();
    let posts = t.posts();
    assert_eq!(posts.len(), 1);
    let question = posts[0].body.as_ref().unwrap();
    assert_eq!(question["action"]["name"], "access");
    assert_eq!(question["resource"]["type"], "service");
    assert_eq!(question["resource"]["id"], SERVICE);
    assert!(
        question
            .pointer("/resource/properties/ancestors/0")
            .is_none()
    );
    assert!(question.pointer("/context/usage").is_none());
    assert!(question.pointer("/context/approval").is_none());
    // A `false` answer gives 403, and a reason of 17.1 is an answer that is not valid: 503.
    for (answer, status) in [
        (json!({"decision": false}), 403),
        (
            json!({"decision": false, "context": {"reason": "approval_required"}}),
            503,
        ),
    ] {
        let reply = answer.clone();
        let (pep, who) = service(Scripted::answering(move |_| reply.clone()), |_| {});
        let refusal = pep.decide_route_without_action(&who).unwrap_err();
        assert_eq!(refusal.to_http().status, status, "{answer}");
    }
}

#[test]
fn p_5_5a_a_list_of_other_permissions_goes_in_one_request_with_permission_only() {
    use procurauth_pep::operation::Permission;
    let t = yes_to_all();
    let (pep, who) = service(t.clone(), |_| {});
    let others = vec![
        Permission::new("data.read", table("test1", "orders")),
        Permission::new("data.read", table("test1", "customers")),
    ];
    let copy = Operation::new("data.write", database("test2")).also_all(others);
    let allowed = pep.decide(&who, &copy).into_result().unwrap();
    assert_eq!(allowed.others().len(), 2);
    let posts = t.posts();
    // One Access Evaluations request with the two other permissions, then the use.
    let batch = posts[0].body.as_ref().unwrap();
    assert_eq!(batch["evaluations"].as_array().unwrap().len(), 2);
    assert_eq!(batch["context"]["permission_only"], true);
    assert!(batch.pointer("/context/usage").is_none());
    let used = posts[1].body.as_ref().unwrap();
    assert_eq!(used["action"]["name"], "data.write");
    assert!(used.pointer("/context/permission_only").is_none());
}

#[test]
fn an_identity_from_a_checked_token_asks_as_the_identity_of_authenticate() {
    let t = yes_to_all();
    let (pep, who) = service(t.clone(), |_| {});
    let again = Identity::from_checked_token(
        who.sub(),
        who.owner().map(str::to_owned),
        who.issued_at(),
        who.expires_at(),
    );
    assert_eq!(again, who);
    let op = Operation::new("data.write", database("test2"));
    pep.decide(&again, &op).into_result().unwrap();
    assert_eq!(
        t.posts()[0].body.as_ref().unwrap()["subject"]["id"],
        "alice"
    );
}
