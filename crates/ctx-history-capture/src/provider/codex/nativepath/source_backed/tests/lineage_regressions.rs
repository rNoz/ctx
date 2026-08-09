use super::*;
use ctx_history_core::RepositoryAbstentionReason;
use serde_json::Value;

use super::projection::{
    exec_call, initialize_repository, outcome_for_sequence, successful_result,
};

fn uuid_v7_at_unix_ms(unix_ms: u64, suffix: u64) -> String {
    format!(
        "{:08x}-{:04x}-7000-8000-{:012x}",
        unix_ms >> 16,
        unix_ms & 0xffff,
        suffix
    )
}

fn assert_child_outcome_is_unproven(index: &VerifiedIndex, child_native_session_id: &str) {
    assert_child_outcome_at_sequence_is_unproven(index, child_native_session_id, 2);
}

fn assert_child_outcome_at_sequence_is_unproven(
    index: &VerifiedIndex,
    child_native_session_id: &str,
    sequence: u64,
) {
    let source = codex_source_key(child_native_session_id).unwrap();
    let session = codex_session_identity(&source, child_native_session_id).unwrap();
    let result = outcome_for_sequence(index, session, sequence);
    assert!(result.repository_vcs_observations.is_empty());
    assert!(result.repository_abstentions.iter().any(|abstention| {
        abstention.reason == RepositoryAbstentionReason::ProviderOutputUnjoined
            && abstention.detail.as_deref() == Some("provider_execution_origin_lineage_unproven")
    }));
}

#[test]
fn post_fork_ancestor_corruption_does_not_poison_a_transitive_child_outcome() {
    use ctx_history_core::EventOrigin;

    let temp = tempfile::tempdir().unwrap();
    let sessions = temp.path().join("sessions");
    let index = temp.path().join("global-index");
    let repository = temp.path().join("repo");
    fs::create_dir_all(&sessions).unwrap();
    initialize_repository(&repository);
    let root = uuid_v7_at_unix_ms(1_785_240_000_000, 1);
    let fork = uuid_v7_at_unix_ms(1_785_241_800_000, 2);
    let child = uuid_v7_at_unix_ms(1_785_242_700_000, 3);
    write_session(
        &sessions,
        &root,
        &[
            message("user", "root before fork"),
            descendant_started(&fork),
            r#"{"timestamp":"2026-07-28T13:00:00Z","type":"response_item","payload":{"type":"function_call","arguments":"#.to_owned(),
        ],
    );
    write_forked_session_at(
        &sessions,
        &fork,
        &root,
        "2026-07-28T12:30:00Z",
        &[
            message("user", "fork before delegated child"),
            descendant_started(&child),
        ],
    );
    write_forked_session_at(
        &sessions,
        &child,
        &fork,
        "2026-07-28T12:45:00Z",
        &[
            exec_call(
                "transitive-child-call",
                "git commit -m child && git rev-parse --verify HEAD",
                &repository,
            ),
            successful_result(
                "transitive-child-call",
                Value::String(
                    "[main 5555555] child\n5555555555555555555555555555555555555555\n".to_owned(),
                ),
            ),
        ],
    );

    ingest_codex_source_backed_v0(&sessions, &index).unwrap();
    let child_source = codex_source_key(&child).unwrap();
    let child_session = codex_session_identity(&child_source, &child).unwrap();
    let verified = VerifiedIndex::open(&index).unwrap();
    let result = outcome_for_sequence(&verified, child_session, 2);
    assert_eq!(result.event_origin, EventOrigin::UniqueToSession);
    assert_eq!(result.repository_vcs_observations.len(), 1);
}

#[test]
fn pre_fork_ancestor_corruption_still_fails_closed_transitively() {
    let temp = tempfile::tempdir().unwrap();
    let sessions = temp.path().join("sessions");
    let index = temp.path().join("global-index");
    let repository = temp.path().join("repo");
    fs::create_dir_all(&sessions).unwrap();
    initialize_repository(&repository);
    let root = uuid_v7_at_unix_ms(1_785_240_000_000, 4);
    let fork = uuid_v7_at_unix_ms(1_785_241_800_000, 5);
    let child = uuid_v7_at_unix_ms(1_785_242_700_000, 6);
    write_session(
        &sessions,
        &root,
        &[
            message("user", "root before fork"),
            r#"{"timestamp":"2026-07-28T12:15:00Z","type":"response_item","payload":{"type":"function_call","arguments":"#.to_owned(),
            descendant_started(&fork),
        ],
    );
    write_forked_session_at(
        &sessions,
        &fork,
        &root,
        "2026-07-28T12:30:00Z",
        &[
            message("user", "fork before delegated child"),
            descendant_started(&child),
        ],
    );
    write_forked_session_at(
        &sessions,
        &child,
        &fork,
        "2026-07-28T12:45:00Z",
        &[
            exec_call(
                "transitive-unproven-call",
                "git commit -m child && git rev-parse --verify HEAD",
                &repository,
            ),
            successful_result(
                "transitive-unproven-call",
                Value::String(
                    "[main 6666666] child\n6666666666666666666666666666666666666666\n".to_owned(),
                ),
            ),
        ],
    );

    ingest_codex_source_backed_v0(&sessions, &index).unwrap();
    assert_child_outcome_is_unproven(&VerifiedIndex::open(&index).unwrap(), &child);
}

#[test]
fn checkpoint_replay_preserves_incomplete_tail_lineage_ambiguity() {
    let temp = tempfile::tempdir().unwrap();
    let sessions = temp.path().join("sessions");
    let index = temp.path().join("global-index");
    let repository = temp.path().join("repo");
    fs::create_dir_all(&sessions).unwrap();
    initialize_repository(&repository);
    let parent = "019fa000-0000-7000-8000-000000000200";
    let child = "019fa000-0000-7000-8000-000000000201";
    let incomplete =
        r#"{"type":"response_item","payload":{"type":"function_call","call_id":"unterminated"#;
    fs::write(
        session_path(&sessions, parent),
        format!(
            "{}\n{}\n{incomplete}",
            session_meta(parent),
            message("user", "parent session anchor")
        ),
    )
    .unwrap();

    ingest_codex_source_backed_v0(&sessions, &index).unwrap();
    write_forked_session(
        &sessions,
        child,
        parent,
        &[
            exec_call(
                "child-after-incomplete-tail",
                "git commit -m child && git rev-parse --verify HEAD",
                &repository,
            ),
            successful_result(
                "child-after-incomplete-tail",
                Value::String(
                    "[main ccccccc] child\ncccccccccccccccccccccccccccccccccccccccc\n".to_owned(),
                ),
            ),
        ],
    );

    let refresh = ingest_codex_source_backed_v0(&sessions, &index).unwrap();
    assert_eq!(refresh.counters.replayed_sources, 1);
    assert_child_outcome_is_unproven(&VerifiedIndex::open(&index).unwrap(), child);
}

#[test]
fn checkpoint_replay_bounds_incomplete_tail_after_typed_descendant_start() {
    use ctx_history_core::EventOrigin;

    let temp = tempfile::tempdir().unwrap();
    let sessions = temp.path().join("sessions");
    let index = temp.path().join("global-index");
    let repository = temp.path().join("repo");
    fs::create_dir_all(&sessions).unwrap();
    initialize_repository(&repository);
    let parent = "019fa000-0000-7000-8000-000000000220";
    let child = "019fa000-0000-7000-8000-000000000221";
    let incomplete =
        r#"{"type":"response_item","payload":{"type":"function_call","call_id":"unterminated"#;
    fs::write(
        session_path(&sessions, parent),
        format!(
            "{}\n{}\n{}\n{incomplete}",
            session_meta(parent),
            message("user", "parent session anchor"),
            descendant_started(child)
        ),
    )
    .unwrap();

    ingest_codex_source_backed_v0(&sessions, &index).unwrap();
    write_forked_session(
        &sessions,
        child,
        parent,
        &[
            exec_call(
                "child-after-bounded-tail",
                "git commit -m child && git rev-parse --verify HEAD",
                &repository,
            ),
            successful_result(
                "child-after-bounded-tail",
                Value::String(
                    "[main ababab1] child\nababab1ababab1ababab1ababab1ababab1ababa\n".to_owned(),
                ),
            ),
        ],
    );

    let refresh = ingest_codex_source_backed_v0(&sessions, &index).unwrap();
    assert_eq!(refresh.counters.replayed_sources, 1);
    let child_source = codex_source_key(child).unwrap();
    let child_session = codex_session_identity(&child_source, child).unwrap();
    let result = outcome_for_sequence(&VerifiedIndex::open(&index).unwrap(), child_session, 2);
    assert_eq!(result.event_origin, EventOrigin::UniqueToSession);
    assert_eq!(result.repository_vcs_observations.len(), 1);
}

#[test]
fn fully_escaped_duplicate_lineage_fields_cannot_publish_a_unique_child_outcome() {
    let temp = tempfile::tempdir().unwrap();
    let sessions = temp.path().join("sessions");
    let index = temp.path().join("global-index");
    let repository = temp.path().join("repo");
    fs::create_dir_all(&sessions).unwrap();
    initialize_repository(&repository);
    let parent = "019fa000-0000-7000-8000-000000000202";
    let child = "019fa000-0000-7000-8000-000000000203";
    let malformed = r#"{"\u0074\u0079\u0070\u0065":"\u0072\u0065\u0073\u0070\u006f\u006e\u0073\u0065\u005f\u0069\u0074\u0065\u006d","\u0070\u0061\u0079\u006c\u006f\u0061\u0064":{"\u0074\u0079\u0070\u0065":"\u0066\u0075\u006e\u0063\u0074\u0069\u006f\u006e\u005f\u0063\u0061\u006c\u006c","\u0063\u0061\u006c\u006c\u005f\u0069\u0064":"first","\u0063\u0061\u006c\u006c\u005f\u0069\u0064":"second"}}"#;
    write_session(
        &sessions,
        parent,
        &[
            message("user", "parent session anchor"),
            malformed.to_owned(),
        ],
    );
    write_forked_session(
        &sessions,
        child,
        parent,
        &[
            exec_call(
                "first",
                "git commit -m child && git rev-parse --verify HEAD",
                &repository,
            ),
            successful_result(
                "first",
                Value::String(
                    "[main ddddddd] child\ndddddddddddddddddddddddddddddddddddddddd\n".to_owned(),
                ),
            ),
        ],
    );

    ingest_codex_source_backed_v0(&sessions, &index).unwrap();
    let verified = VerifiedIndex::open(&index).unwrap();
    let parent_source = codex_source_key(parent).unwrap();
    let parent_certificate = verified
        .manifest()
        .sources
        .iter()
        .find(|certificate| {
            certificate
                .observation()
                .source()
                .exact_descriptor_eq(&parent_source)
        })
        .unwrap();
    assert_eq!(parent_certificate.counts().rejected_records, 1);
    assert_child_outcome_is_unproven(&verified, child);
}

#[test]
fn escaped_duplicate_envelope_type_with_non_string_first_cannot_hide_ancestor_call() {
    let temp = tempfile::tempdir().unwrap();
    let sessions = temp.path().join("sessions");
    let index = temp.path().join("global-index");
    let repository = temp.path().join("repo");
    fs::create_dir_all(&sessions).unwrap();
    initialize_repository(&repository);
    let parent = "019fa000-0000-7000-8000-000000000209";
    let child = "019fa000-0000-7000-8000-000000000210";
    let call_id = "escaped-duplicate-envelope-type";
    let malformed = format!(
        r#"{{"\u0074\u0079\u0070\u0065":{{}},"\u0074\u0079\u0070\u0065":"\u0072\u0065\u0073\u0070\u006f\u006e\u0073\u0065\u005f\u0069\u0074\u0065\u006d","\u0070\u0061\u0079\u006c\u006f\u0061\u0064":{{"\u0074\u0079\u0070\u0065":"\u0066\u0075\u006e\u0063\u0074\u0069\u006f\u006e\u005f\u0063\u0061\u006c\u006c","\u0063\u0061\u006c\u006c\u005f\u0069\u0064":"{call_id}"}}}}"#
    );
    write_session(
        &sessions,
        parent,
        &[message("user", "parent session anchor"), malformed],
    );
    write_forked_session(
        &sessions,
        child,
        parent,
        &[
            exec_call(
                call_id,
                "git commit -m child && git rev-parse --verify HEAD",
                &repository,
            ),
            successful_result(
                call_id,
                Value::String(
                    "[main 1111111] child\n1111111111111111111111111111111111111111\n".to_owned(),
                ),
            ),
        ],
    );

    ingest_codex_source_backed_v0(&sessions, &index).unwrap();
    assert_child_outcome_is_unproven(&VerifiedIndex::open(&index).unwrap(), child);
}

#[test]
fn escaped_duplicate_payload_type_with_non_string_first_cannot_hide_ancestor_result() {
    let temp = tempfile::tempdir().unwrap();
    let sessions = temp.path().join("sessions");
    let index = temp.path().join("global-index");
    let repository = temp.path().join("repo");
    fs::create_dir_all(&sessions).unwrap();
    initialize_repository(&repository);
    let parent = "019fa000-0000-7000-8000-000000000211";
    let child = "019fa000-0000-7000-8000-000000000212";
    let call_id = "escaped-duplicate-payload-type";
    let malformed = format!(
        r#"{{"\u0074\u0079\u0070\u0065":"\u0072\u0065\u0073\u0070\u006f\u006e\u0073\u0065\u005f\u0069\u0074\u0065\u006d","\u0070\u0061\u0079\u006c\u006f\u0061\u0064":{{"\u0074\u0079\u0070\u0065":[],"\u0074\u0079\u0070\u0065":"\u0066\u0075\u006e\u0063\u0074\u0069\u006f\u006e\u005f\u0063\u0061\u006c\u006c\u005f\u006f\u0075\u0074\u0070\u0075\u0074","\u0063\u0061\u006c\u006c\u005f\u0069\u0064":"{call_id}","output":"ancestor"}}}}"#
    );
    write_session(
        &sessions,
        parent,
        &[message("user", "parent session anchor"), malformed],
    );
    write_forked_session(
        &sessions,
        child,
        parent,
        &[
            exec_call(
                call_id,
                "git commit -m child && git rev-parse --verify HEAD",
                &repository,
            ),
            successful_result(
                call_id,
                Value::String(
                    "[main 2222222] child\n2222222222222222222222222222222222222222\n".to_owned(),
                ),
            ),
        ],
    );

    ingest_codex_source_backed_v0(&sessions, &index).unwrap();
    assert_child_outcome_is_unproven(&VerifiedIndex::open(&index).unwrap(), child);
}

#[test]
fn attributed_malformed_call_does_not_suppress_an_unrelated_unique_call() {
    let temp = tempfile::tempdir().unwrap();
    let sessions = temp.path().join("sessions");
    let index = temp.path().join("global-index");
    let repository = temp.path().join("repo");
    fs::create_dir_all(&sessions).unwrap();
    initialize_repository(&repository);
    let parent = "019fa000-0000-7000-8000-000000000213";
    let child = "019fa000-0000-7000-8000-000000000214";
    let ambiguous_call = "malformed-call-a";
    let unique_call = "unrelated-call-b";
    let malformed = format!(
        r#"{{"type":"response_item","payload":{{"type":null,"type":"function_call","call_id":"{ambiguous_call}"}}}}"#
    );
    write_session(
        &sessions,
        parent,
        &[message("user", "parent session anchor"), malformed],
    );
    write_forked_session(
        &sessions,
        child,
        parent,
        &[
            exec_call(
                ambiguous_call,
                "git commit -m ambiguous && git rev-parse --verify HEAD",
                &repository,
            ),
            successful_result(
                ambiguous_call,
                Value::String(
                    "[main 3333333] ambiguous\n3333333333333333333333333333333333333333\n"
                        .to_owned(),
                ),
            ),
            exec_call(
                unique_call,
                "git commit -m unique && git rev-parse --verify HEAD",
                &repository,
            ),
            successful_result(
                unique_call,
                Value::String(
                    "[main 4444444] unique\n4444444444444444444444444444444444444444\n".to_owned(),
                ),
            ),
        ],
    );

    ingest_codex_source_backed_v0(&sessions, &index).unwrap();
    let verified = VerifiedIndex::open(&index).unwrap();
    assert_child_outcome_at_sequence_is_unproven(&verified, child, 2);
    let child_source = codex_source_key(child).unwrap();
    let child_session = codex_session_identity(&child_source, child).unwrap();
    let unique = outcome_for_sequence(&verified, child_session, 4);
    assert_eq!(unique.repository_vcs_observations.len(), 1);
    assert!(!unique.repository_abstentions.iter().any(|abstention| {
        abstention.reason == RepositoryAbstentionReason::ProviderOutputUnjoined
            && abstention.detail.as_deref() == Some("provider_execution_origin_lineage_unproven")
    }));
}

#[test]
fn fully_escaped_duplicate_call_id_after_non_string_cannot_publish_a_unique_child_outcome() {
    let temp = tempfile::tempdir().unwrap();
    let sessions = temp.path().join("sessions");
    let index = temp.path().join("global-index");
    let repository = temp.path().join("repo");
    fs::create_dir_all(&sessions).unwrap();
    initialize_repository(&repository);
    let parent = "019fa000-0000-7000-8000-000000000205";
    let child = "019fa000-0000-7000-8000-000000000206";
    let call_id = "escaped-after-non-string";
    let malformed = format!(
        r#"{{"\u0074\u0079\u0070\u0065":"\u0072\u0065\u0073\u0070\u006f\u006e\u0073\u0065\u005f\u0069\u0074\u0065\u006d","\u0070\u0061\u0079\u006c\u006f\u0061\u0064":{{"\u0074\u0079\u0070\u0065":"\u0066\u0075\u006e\u0063\u0074\u0069\u006f\u006e\u005f\u0063\u0061\u006c\u006c","\u0063\u0061\u006c\u006c\u005f\u0069\u0064":7,"\u0063\u0061\u006c\u006c\u005f\u0069\u0064":"{call_id}"}}}}"#
    );
    write_session(
        &sessions,
        parent,
        &[message("user", "parent session anchor"), malformed],
    );
    write_forked_session(
        &sessions,
        child,
        parent,
        &[
            exec_call(
                call_id,
                "git commit -m child && git rev-parse --verify HEAD",
                &repository,
            ),
            successful_result(
                call_id,
                Value::String(
                    "[main eeeeeee] child\neeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee\n".to_owned(),
                ),
            ),
        ],
    );

    ingest_codex_source_backed_v0(&sessions, &index).unwrap();
    assert_child_outcome_is_unproven(&VerifiedIndex::open(&index).unwrap(), child);
}

#[test]
fn completed_parent_checkpoint_tail_rebuilds_child_and_replays_without_stale_ambiguity() {
    let temp = tempfile::tempdir().unwrap();
    let sessions = temp.path().join("sessions");
    let index = temp.path().join("global-index");
    let repository = temp.path().join("repo");
    fs::create_dir_all(&sessions).unwrap();
    initialize_repository(&repository);
    let parent = "019fa000-0000-7000-8000-000000000207";
    let child = "019fa000-0000-7000-8000-000000000208";
    let call_id = "unique-after-tail-completion";
    let incomplete = r#"{"type":"event_msg","payload":{"type":"token_count""#;
    let parent_path = session_path(&sessions, parent);
    fs::write(
        &parent_path,
        format!(
            "{}\n{}\n{incomplete}",
            session_meta(parent),
            message("user", "parent session anchor")
        ),
    )
    .unwrap();
    write_forked_session(
        &sessions,
        child,
        parent,
        &[
            exec_call(
                call_id,
                "git commit -m child && git rev-parse --verify HEAD",
                &repository,
            ),
            successful_result(
                call_id,
                Value::String(
                    "[main fffffff] child\nffffffffffffffffffffffffffffffffffffffff\n".to_owned(),
                ),
            ),
        ],
    );

    ingest_codex_source_backed_v0(&sessions, &index).unwrap();
    assert_child_outcome_is_unproven(&VerifiedIndex::open(&index).unwrap(), child);

    use std::io::Write;
    writeln!(
        fs::OpenOptions::new()
            .append(true)
            .open(&parent_path)
            .unwrap(),
        "}}}}"
    )
    .unwrap();
    let completed = ingest_codex_source_backed_v0(&sessions, &index).unwrap();
    assert_eq!(completed.counters.appended_sources, 1);
    assert_eq!(completed.counters.replaced_sources, 1);
    let verified = VerifiedIndex::open(&index).unwrap();
    let child_source = codex_source_key(child).unwrap();
    let child_session = codex_session_identity(&child_source, child).unwrap();
    let result = outcome_for_sequence(&verified, child_session, 2);
    assert_eq!(result.repository_vcs_observations.len(), 1);
    assert!(!result.repository_abstentions.iter().any(|abstention| {
        abstention.reason == RepositoryAbstentionReason::ProviderOutputUnjoined
            && abstention.detail.as_deref() == Some("provider_execution_origin_lineage_unproven")
    }));

    let replay = ingest_codex_source_backed_v0(&sessions, &index).unwrap();
    assert_eq!(replay.counters.replayed_sources, 2);
    assert_eq!(replay.commit.generation_id, completed.commit.generation_id);
    let replayed = VerifiedIndex::open(&index).unwrap();
    assert_eq!(
        outcome_for_sequence(&replayed, child_session, 2)
            .repository_vcs_observations
            .len(),
        1
    );
}
