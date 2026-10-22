// Ported from ink test/input-parser.ts @ 26d2c3f.
// String chunks enter through the UTF-8 byte boundary; event expectations are unchanged.

use super::*;

fn parse_chunks(chunks: &[&str]) -> Vec<InputEvent> {
    let mut parser = InputParser::default();
    chunks
        .iter()
        .flat_map(|chunk| parser.push(chunk.as_bytes()))
        .collect()
}

#[test]
fn passes_through_plain_text_chunks() {
    assert_eq!(
        parse_chunks(&["hello", " ", "world"]),
        vec![
            InputEvent::Text("hello".into()),
            InputEvent::Text(" ".into()),
            InputEvent::Text("world".into())
        ]
    );
}

#[test]
fn keeps_plain_text_and_control_sequences_separate() {
    assert_eq!(
        parse_chunks(&["a\u{1b}[Ab"]),
        vec![
            InputEvent::Text("a".into()),
            InputEvent::Text("\u{1b}[A".into()),
            InputEvent::Text("b".into())
        ]
    );
}

#[test]
fn parses_multiple_standard_csi_keys_in_one_chunk() {
    assert_eq!(
        parse_chunks(&["\u{1b}[A\u{1b}[B\u{1b}[C\u{1b}[D"]),
        vec![
            InputEvent::Text("\u{1b}[A".into()),
            InputEvent::Text("\u{1b}[B".into()),
            InputEvent::Text("\u{1b}[C".into()),
            InputEvent::Text("\u{1b}[D".into())
        ]
    );
}

#[test]
fn parses_csi_sequences_with_parameters() {
    assert_eq!(
        parse_chunks(&["\u{1b}[1;5A\u{1b}[5~\u{1b}[6~"]),
        vec![
            InputEvent::Text("\u{1b}[1;5A".into()),
            InputEvent::Text("\u{1b}[5~".into()),
            InputEvent::Text("\u{1b}[6~".into())
        ]
    );
}

#[test]
fn keeps_rxvt_shifted_editing_keys_separate_from_following_text() {
    for key in [2, 3, 5, 6, 7, 8] {
        let sequence = format!("\u{1b}[{key}$");

        assert_eq!(
            parse_chunks(&[&format!("{sequence}hello")]),
            vec![InputEvent::Text(sequence), InputEvent::Text("hello".into())]
        );
    }
}

#[test]
fn emits_rxvt_shift_delete_as_soon_as_its_final_byte_arrives() {
    let mut parser = InputParser::default();

    assert_eq!(parser.push("\u{1b}[3".as_bytes()), vec![]);
    assert_eq!(
        parser.push("$".as_bytes()),
        vec![InputEvent::Text("\u{1b}[3$".into())]
    );
    assert!(!parser.has_pending_escape());
    assert_eq!(
        parser.push("hello".as_bytes()),
        vec![InputEvent::Text("hello".into())]
    );
}

#[test]
fn parses_consecutive_rxvt_shifted_keys_including_meta() {
    assert_eq!(
        parse_chunks(&["\u{1b}[3$\u{1b}\u{1b}[5$\u{1b}[A"]),
        vec![
            InputEvent::Text("\u{1b}[3$".into()),
            InputEvent::Text("\u{1b}\u{1b}[5$".into()),
            InputEvent::Text("\u{1b}[A".into())
        ]
    );
}

#[test]
fn preserves_csi_intermediate_bytes_outside_rxvt_shifted_keys() {
    assert_eq!(
        parse_chunks(&["\u{1b}[1;", "2$", "yhello"]),
        vec![
            InputEvent::Text("\u{1b}[1;2$y".into()),
            InputEvent::Text("hello".into())
        ]
    );
}

#[test]
fn parses_kitty_protocol_sequence_as_one_key_event() {
    assert_eq!(
        parse_chunks(&["\u{1b}[97;5u"]),
        vec![InputEvent::Text("\u{1b}[97;5u".into())]
    );
}

#[test]
fn parses_ss3_sequences_as_one_key_event() {
    assert_eq!(
        parse_chunks(&["\u{1b}OA\u{1b}OB\u{1b}OC\u{1b}OD"]),
        vec![
            InputEvent::Text("\u{1b}OA".into()),
            InputEvent::Text("\u{1b}OB".into()),
            InputEvent::Text("\u{1b}OC".into()),
            InputEvent::Text("\u{1b}OD".into())
        ]
    );
}

#[test]
fn preserves_modified_ss3_keys_and_following_text() {
    for sequence in ["\u{1b}O2P", "\u{1b}O1;5A", "\u{1b}\u{1b}O5D"] {
        assert_eq!(
            parse_chunks(&[&format!("{sequence}hello")]),
            vec![
                InputEvent::Text(sequence.into()),
                InputEvent::Text("hello".into())
            ]
        );
    }
}

#[test]
fn holds_modified_ss3_sequences_until_their_final_byte_arrives() {
    let mut parser = InputParser::default();

    assert_eq!(parser.push("\u{1b}O1".as_bytes()), vec![]);
    assert!(parser.has_pending_escape());
    assert_eq!(parser.push(";5".as_bytes()), vec![]);
    assert_eq!(
        parser.push("A".as_bytes()),
        vec![InputEvent::Text("\u{1b}O1;5A".into())]
    );
    assert!(!parser.has_pending_escape());
}

#[test]
fn does_not_consume_a_following_escape_as_ss3_final_byte() {
    assert_eq!(
        parse_chunks(&["\u{1b}O\u{1b}[A"]),
        vec![
            InputEvent::Text("\u{1b}O".into()),
            InputEvent::Text("\u{1b}[A".into())
        ]
    );
}

#[test]
fn parses_meta_csi_sequence_with_double_escape() {
    assert_eq!(
        parse_chunks(&["\u{1b}\u{1b}[A"]),
        vec![InputEvent::Text("\u{1b}\u{1b}[A".into())]
    );
}

#[test]
fn parses_escaped_printable_code_points() {
    assert_eq!(
        parse_chunks(&["\u{1b}x\u{1b}1"]),
        vec![
            InputEvent::Text("\u{1b}x".into()),
            InputEvent::Text("\u{1b}1".into())
        ]
    );
}

#[test]
fn parses_escaped_supplementary_code_points() {
    assert_eq!(
        parse_chunks(&["\u{1b}\u{1f600}"]),
        vec![InputEvent::Text("\u{1b}\u{1f600}".into())]
    );
}

#[test]
fn preserves_legacy_esc_sequences_in_a_mixed_chunk() {
    assert_eq!(
        parse_chunks(&["\u{1b}[[A\u{1b}[[5~"]),
        vec![
            InputEvent::Text("\u{1b}[[A".into()),
            InputEvent::Text("\u{1b}[[5~".into())
        ]
    );
}

#[test]
fn preserves_legacy_esc_sequences_across_chunks() {
    assert_eq!(
        parse_chunks(&["\u{1b}[[", "A\u{1b}[[5~"]),
        vec![
            InputEvent::Text("\u{1b}[[A".into()),
            InputEvent::Text("\u{1b}[[5~".into())
        ]
    );
}

#[test]
fn parses_legacy_and_standard_csi_sequences_mixed_together() {
    assert_eq!(
        parse_chunks(&["\u{1b}[[A\u{1b}[B\u{1b}[[6~\u{1b}[1;5D"]),
        vec![
            InputEvent::Text("\u{1b}[[A".into()),
            InputEvent::Text("\u{1b}[B".into()),
            InputEvent::Text("\u{1b}[[6~".into()),
            InputEvent::Text("\u{1b}[1;5D".into())
        ]
    );
}

#[test]
fn holds_incomplete_csi_sequence_until_final_byte_arrives() {
    let mut parser = InputParser::default();

    assert_eq!(parser.push("\u{1b}[".as_bytes()), vec![]);
    assert!(parser.has_pending_escape());
    assert_eq!(parser.push("1;5".as_bytes()), vec![]);
    assert_eq!(
        parser.push("A".as_bytes()),
        vec![InputEvent::Text("\u{1b}[1;5A".into())]
    );
}

#[test]
fn holds_incomplete_legacy_esc_sequence_until_final_byte_arrives() {
    let mut parser = InputParser::default();

    assert_eq!(parser.push("\u{1b}[[".as_bytes()), vec![]);
    assert_eq!(parser.push("5".as_bytes()), vec![]);
    assert_eq!(
        parser.push("~".as_bytes()),
        vec![InputEvent::Text("\u{1b}[[5~".into())]
    );
}

#[test]
fn holds_incomplete_ss3_sequence_until_final_byte_arrives() {
    let mut parser = InputParser::default();

    assert_eq!(parser.push("\u{1b}O".as_bytes()), vec![]);
    assert_eq!(
        parser.push("A".as_bytes()),
        vec![InputEvent::Text("\u{1b}OA".into())]
    );
}

#[test]
fn holds_incomplete_double_escape_csi_sequence_until_final_byte_arrives() {
    let mut parser = InputParser::default();

    assert_eq!(parser.push("\u{1b}\u{1b}[".as_bytes()), vec![]);
    assert_eq!(
        parser.push("A".as_bytes()),
        vec![InputEvent::Text("\u{1b}\u{1b}[A".into())]
    );
}

#[test]
fn keeps_pending_plain_escape_and_can_flush_it() {
    let mut parser = InputParser::default();

    assert_eq!(parser.push("\u{1b}".as_bytes()), vec![]);
    assert!(parser.has_pending_escape());
    assert_eq!(parser.flush_pending_escape(), Some("\u{1b}".into()));
    assert!(!parser.has_pending_escape());
}

#[test]
fn flushes_pending_csi_prefix_as_literal_input() {
    let mut parser = InputParser::default();

    assert_eq!(parser.push("\u{1b}[".as_bytes()), vec![]);
    assert!(parser.has_pending_escape());
    assert_eq!(parser.flush_pending_escape(), Some("\u{1b}[".into()));
    assert!(!parser.has_pending_escape());
    assert_eq!(
        parser.push("A".as_bytes()),
        vec![InputEvent::Text("A".into())]
    );
}

#[test]
fn reset_clears_pending_input_state() {
    let mut parser = InputParser::default();

    assert_eq!(parser.push("\u{1b}[".as_bytes()), vec![]);
    parser.reset();
    assert_eq!(
        parser.push("A".as_bytes()),
        vec![InputEvent::Text("A".into())]
    );
}

#[test]
fn treats_invalid_csi_continuation_as_escaped_code_point_plus_plain_text() {
    assert_eq!(
        parse_chunks(&["\u{1b}[\n"]),
        vec![
            InputEvent::Text("\u{1b}[".into()),
            InputEvent::Text("\n".into())
        ]
    );
}

#[test]
fn parses_mixed_text_and_many_key_events_in_one_read() {
    assert_eq!(
        parse_chunks(&["start\u{1b}[A mid \u{1b}OH end\u{1b}[[5~"]),
        vec![
            InputEvent::Text("start".into()),
            InputEvent::Text("\u{1b}[A".into()),
            InputEvent::Text(" mid ".into()),
            InputEvent::Text("\u{1b}OH".into()),
            InputEvent::Text(" end".into()),
            InputEvent::Text("\u{1b}[[5~".into())
        ]
    );
}

#[test]
fn flushes_pending_ss3_prefix_as_literal_input() {
    let mut parser = InputParser::default();

    assert_eq!(parser.push("\u{1b}O".as_bytes()), vec![]);
    assert!(parser.has_pending_escape());
    assert_eq!(parser.flush_pending_escape(), Some("\u{1b}O".into()));
    assert_eq!(
        parser.push("x".as_bytes()),
        vec![InputEvent::Text("x".into())]
    );
}

#[test]
fn flushes_pending_legacy_csi_prefix_as_literal_input() {
    let mut parser = InputParser::default();

    assert_eq!(parser.push("\u{1b}[[".as_bytes()), vec![]);
    assert!(parser.has_pending_escape());
    assert_eq!(parser.flush_pending_escape(), Some("\u{1b}[[".into()));
    assert_eq!(
        parser.push("x".as_bytes()),
        vec![InputEvent::Text("x".into())]
    );
}

#[test]
fn parses_meta_ss3_sequence_with_double_escape() {
    assert_eq!(
        parse_chunks(&["\u{1b}\u{1b}OA"]),
        vec![InputEvent::Text("\u{1b}\u{1b}OA".into())]
    );
}

#[test]
fn holds_incomplete_double_escape_ss3_sequence_until_final_byte_arrives() {
    let mut parser = InputParser::default();

    assert_eq!(parser.push("\u{1b}\u{1b}O".as_bytes()), vec![]);
    assert!(parser.has_pending_escape());
    assert_eq!(
        parser.push("A".as_bytes()),
        vec![InputEvent::Text("\u{1b}\u{1b}OA".into())]
    );
}

#[test]
fn emits_double_escape_as_single_event_for_non_control_character() {
    assert_eq!(
        parse_chunks(&["\u{1b}\u{1b}x"]),
        vec![
            InputEvent::Text("\u{1b}\u{1b}".into()),
            InputEvent::Text("x".into())
        ]
    );
}

#[test]
fn empty_chunk_produces_no_events() {
    assert_eq!(parse_chunks(&[""]), vec![]);
}

#[test]
fn empty_chunk_does_not_disturb_pending_state() {
    let mut parser = InputParser::default();

    assert_eq!(parser.push("\u{1b}[".as_bytes()), vec![]);
    assert_eq!(parser.push("".as_bytes()), vec![]);
    assert!(parser.has_pending_escape());
    assert_eq!(
        parser.push("A".as_bytes()),
        vec![InputEvent::Text("\u{1b}[A".into())]
    );
}

#[test]
fn plain_text_followed_by_incomplete_escape_holds_escape_as_pending() {
    let mut parser = InputParser::default();

    assert_eq!(
        parser.push("hello\u{1b}".as_bytes()),
        vec![InputEvent::Text("hello".into())]
    );
    assert!(parser.has_pending_escape());
    assert_eq!(parser.flush_pending_escape(), Some("\u{1b}".into()));
}

#[test]
fn splits_batched_0x7f_backspace_characters_into_individual_events() {
    assert_eq!(
        parse_chunks(&["\u{7f}\u{7f}\u{7f}"]),
        vec![
            InputEvent::Text("\u{7f}".into()),
            InputEvent::Text("\u{7f}".into()),
            InputEvent::Text("\u{7f}".into())
        ]
    );
}

#[test]
fn splits_batched_backspace_characters_into_individual_events() {
    assert_eq!(
        parse_chunks(&["\u{8}\u{8}\u{8}"]),
        vec![
            InputEvent::Text("\u{8}".into()),
            InputEvent::Text("\u{8}".into()),
            InputEvent::Text("\u{8}".into())
        ]
    );
}

#[test]
fn splits_mixed_0x7f_and_0x08_backspace_characters() {
    assert_eq!(
        parse_chunks(&["\u{7f}\u{8}\u{7f}"]),
        vec![
            InputEvent::Text("\u{7f}".into()),
            InputEvent::Text("\u{8}".into()),
            InputEvent::Text("\u{7f}".into())
        ]
    );
}

#[test]
fn splits_mixed_printable_text_and_0x7f_backspace_characters() {
    assert_eq!(
        parse_chunks(&["abc\u{7f}\u{7f}\u{7f}"]),
        vec![
            InputEvent::Text("abc".into()),
            InputEvent::Text("\u{7f}".into()),
            InputEvent::Text("\u{7f}".into()),
            InputEvent::Text("\u{7f}".into())
        ]
    );
}

#[test]
fn single_0x7f_backspace_character_is_preserved_as_individual_event() {
    assert_eq!(
        parse_chunks(&["\u{7f}"]),
        vec![InputEvent::Text("\u{7f}".into())]
    );
}

#[test]
fn single_backspace_character_is_preserved_as_individual_event() {
    assert_eq!(
        parse_chunks(&["\u{8}"]),
        vec![InputEvent::Text("\u{8}".into())]
    );
}

#[test]
fn splits_trailing_0x7f_backspace_from_text() {
    assert_eq!(
        parse_chunks(&["abc\u{7f}"]),
        vec![
            InputEvent::Text("abc".into()),
            InputEvent::Text("\u{7f}".into())
        ]
    );
}

#[test]
fn splits_0x7f_backspace_characters_before_escape_sequences() {
    assert_eq!(
        parse_chunks(&["\u{7f}\u{7f}\u{1b}[A"]),
        vec![
            InputEvent::Text("\u{7f}".into()),
            InputEvent::Text("\u{7f}".into()),
            InputEvent::Text("\u{1b}[A".into())
        ]
    );
}

#[test]
fn splits_0x7f_backspace_characters_after_escape_sequences() {
    assert_eq!(
        parse_chunks(&["\u{1b}[A\u{7f}\u{7f}"]),
        vec![
            InputEvent::Text("\u{1b}[A".into()),
            InputEvent::Text("\u{7f}".into()),
            InputEvent::Text("\u{7f}".into())
        ]
    );
}

#[test]
fn splits_0x7f_backspace_characters_between_escape_sequences() {
    assert_eq!(
        parse_chunks(&["\u{1b}[A\u{7f}\u{1b}[B"]),
        vec![
            InputEvent::Text("\u{1b}[A".into()),
            InputEvent::Text("\u{7f}".into()),
            InputEvent::Text("\u{1b}[B".into())
        ]
    );
}

#[test]
fn splits_backspace_characters_around_escape_sequences() {
    assert_eq!(
        parse_chunks(&["\u{8}\u{1b}[A\u{8}"]),
        vec![
            InputEvent::Text("\u{8}".into()),
            InputEvent::Text("\u{1b}[A".into()),
            InputEvent::Text("\u{8}".into())
        ]
    );
}

#[test]
fn splits_interleaved_text_and_0x7f_backspace_characters() {
    assert_eq!(
        parse_chunks(&["ab\u{7f}cd"]),
        vec![
            InputEvent::Text("ab".into()),
            InputEvent::Text("\u{7f}".into()),
            InputEvent::Text("cd".into())
        ]
    );
}

// Ink keeps CR and TAB in the text run; ink-rs splits them so keys read
// together are not lost.
#[test]
fn splits_carriage_return_from_text() {
    assert_eq!(
        parse_chunks(&["\rtest"]),
        vec![
            InputEvent::Text("\r".into()),
            InputEvent::Text("test".into())
        ]
    );
}

#[test]
fn splits_tab_from_text() {
    assert_eq!(
        parse_chunks(&["\ttest"]),
        vec![
            InputEvent::Text("\t".into()),
            InputEvent::Text("test".into())
        ]
    );
}

#[test]
fn assembles_csi_sequence_from_single_byte_chunks() {
    let mut parser = InputParser::default();

    assert_eq!(parser.push("\u{1b}".as_bytes()), vec![]);
    assert_eq!(parser.push("[".as_bytes()), vec![]);
    assert_eq!(parser.push("1".as_bytes()), vec![]);
    assert_eq!(parser.push(";".as_bytes()), vec![]);
    assert_eq!(parser.push("5".as_bytes()), vec![]);
    assert_eq!(
        parser.push("A".as_bytes()),
        vec![InputEvent::Text("\u{1b}[1;5A".into())]
    );
}

#[test]
fn emits_paste_event_for_bracketed_paste_sequence() {
    assert_eq!(
        parse_chunks(&["\u{1b}[200~hello world\u{1b}[201~"]),
        vec![InputEvent::Paste("hello world".into())]
    );
}

#[test]
fn emits_paste_event_for_multiline_bracketed_paste() {
    assert_eq!(
        parse_chunks(&["\u{1b}[200~line1\nline2\u{1b}[201~"]),
        vec![InputEvent::Paste("line1\nline2".into())]
    );
}

#[test]
fn paste_content_with_escape_sequences_is_delivered_verbatim() {
    assert_eq!(
        parse_chunks(&["\u{1b}[200~hello\u{1b}[Aworld\u{1b}[201~"]),
        vec![InputEvent::Paste("hello\u{1b}[Aworld".into())]
    );
}

#[test]
fn emits_normal_events_before_and_after_bracketed_paste() {
    assert_eq!(
        parse_chunks(&["before\u{1b}[200~pasted\u{1b}[201~after"]),
        vec![
            InputEvent::Text("before".into()),
            InputEvent::Paste("pasted".into()),
            InputEvent::Text("after".into())
        ]
    );
}

#[test]
fn emits_multiple_paste_events_in_one_chunk() {
    assert_eq!(
        parse_chunks(&["\u{1b}[200~first\u{1b}[201~mid\u{1b}[200~second\u{1b}[201~"]),
        vec![
            InputEvent::Paste("first".into()),
            InputEvent::Text("mid".into()),
            InputEvent::Paste("second".into())
        ]
    );
}

#[test]
fn holds_incomplete_bracketed_paste_as_pending() {
    let mut parser = InputParser::default();

    assert_eq!(parser.push("\u{1b}[200~hello".as_bytes()), vec![]);
    assert!(!parser.has_pending_escape());
    assert_eq!(
        parser.push(" world\u{1b}[201~".as_bytes()),
        vec![InputEvent::Paste("hello world".into())]
    );
}

#[test]
fn assembles_bracketed_paste_from_chunk_by_chunk_delivery() {
    let mut parser = InputParser::default();

    assert_eq!(parser.push("\u{1b}[200~".as_bytes()), vec![]);
    assert_eq!(parser.push("hello".as_bytes()), vec![]);
    assert_eq!(
        parser.push("\u{1b}[201~".as_bytes()),
        vec![InputEvent::Paste("hello".into())]
    );
}

#[test]
fn emits_empty_paste_for_adjacent_paste_markers() {
    assert_eq!(
        parse_chunks(&["\u{1b}[200~\u{1b}[201~"]),
        vec![InputEvent::Paste("".into())]
    );
}

#[test]
fn handles_paste_start_split_before_the_tilde_u001b_200_without() {
    let mut parser = InputParser::default();

    // Chunk ends exactly at the 5th byte of the 6-byte pasteStart sequence.
    // Keep waiting for the final `~` to avoid splitting bracketed paste input.
    assert_eq!(parser.push("\u{1b}[200".as_bytes()), vec![]);
    assert!(!parser.has_pending_escape());
    assert_eq!(
        parser.push("~hello\u{1b}[201~".as_bytes()),
        vec![InputEvent::Paste("hello".into())]
    );
}

#[test]
fn has_pending_escape_returns_true_for_length_3_paste_start_prefix_u001b_2() {
    let mut parser = InputParser::default();

    assert_eq!(parser.push("\u{1b}[2".as_bytes()), vec![]);
    assert!(parser.has_pending_escape());
}

#[test]
fn has_pending_escape_returns_true_for_length_4_paste_start_prefix_u001b_20() {
    let mut parser = InputParser::default();

    assert_eq!(parser.push("\u{1b}[20".as_bytes()), vec![]);
    assert!(parser.has_pending_escape());
}

#[test]
fn paste_event_delivers_backspace_chars_verbatim_without_splitting() {
    assert_eq!(
        parse_chunks(&["\u{1b}[200~\u{7f}\u{8}\u{7f}\u{1b}[201~"]),
        vec![InputEvent::Paste("\u{7f}\u{8}\u{7f}".into())]
    );
}

#[test]
fn is_complete_control_sequence_accepts_every_complete_csi_and_ss3_form_the_parser_emits() {
    for sequence in [
        "\u{1b}[A",
        "\u{1b}[I",
        "\u{1b}[24;80R",
        "\u{1b}[<0;10;20M",
        "\u{1b}[[A",
        "\u{1b}[2$",
        "\u{1b}[ q",
        "\u{1b}OP",
        "\u{1b}O1;5A",
        "\u{1b}\u{1b}[A",
        "\u{1b}\u{1b}O5D",
    ] {
        assert!(
            is_complete_control_sequence(sequence),
            "Sequence: {sequence:?}"
        );
        assert_eq!(
            parse_chunks(&[sequence]),
            vec![InputEvent::Text(sequence.into())]
        );
    }
}

#[test]
fn is_complete_control_sequence_rejects_partial_sequences_and_escaped_code_points() {
    for input in [
        "",
        "q",
        "\u{1b}",
        "\u{1b}[",
        "\u{1b}[1;",
        "\u{1b}[[",
        "\u{1b}O",
        "\u{1b}O1",
        "\u{1b}x",
        "\u{1b}\u{1b}",
        "\u{1b}\u{1b}x",
        "\u{1b}[1$",
        "\u{1b}[Aq",
    ] {
        assert!(!is_complete_control_sequence(input), "Input: {input:?}");
    }
}

// Native regression: KK-71 covers valid split UTF-8, not malformed prefixes.
#[test]
fn invalid_utf8_uses_maximal_subparts_across_reads() {
    let mut parser = InputParser::default();
    assert!(parser.push(&[0xe2, 0x82]).is_empty());
    assert_eq!(
        parser.push(b"x"),
        vec![InputEvent::Text("\u{fffd}x".into())]
    );
    assert_eq!(
        parser.push(&[0xed, 0xa0, 0x80, 0xff, 0xf0, 0x9f]),
        vec![InputEvent::Text("\u{fffd}\u{fffd}\u{fffd}\u{fffd}".into())]
    );
    assert_eq!(
        parser.push(&[0x92, 0xa9]),
        vec![InputEvent::Text("\u{1f4a9}".into())]
    );
    assert!(parser.push(&[0xe2]).is_empty());
    parser.reset();
    assert_eq!(
        parser.push(&[0x82]),
        vec![InputEvent::Text("\u{fffd}".into())]
    );
}

#[test]
fn terminal_strings_have_their_own_events_with_either_terminator() {
    for introducer in ["_", "P", "]", "X", "^"] {
        for terminator in ["\x1b\\", "\x07"] {
            let sequence = format!("\x1b{introducer}Gi=31;OK{terminator}");
            assert_eq!(
                parse_chunks(&[&sequence]),
                vec![InputEvent::StringSequence(sequence)]
            );
        }
    }
}

#[test]
fn terminal_string_and_st_can_span_reads() {
    assert_eq!(
        parse_chunks(&["\x1b", "_Gi=", "31;OK", "\x1b", "\\x"]),
        vec![
            InputEvent::StringSequence("\x1b_Gi=31;OK\x1b\\".into()),
            InputEvent::Text("x".into())
        ]
    );
}

#[test]
fn terminal_string_timeout_preserves_alt_keys_and_the_tail() {
    for introducer in ["P", "_"] {
        let alt = format!("\x1b{introducer}");
        let mut parser = InputParser::default();
        assert!(parser.push(alt.as_bytes()).is_empty());
        assert!(parser.has_pending_escape());
        assert_eq!(parser.flush_pending_escape(), Some(alt.clone()));
        assert_eq!(parser.push(b"x"), vec![InputEvent::Text("x".into())]);
        assert!(parser.push(format!("{alt}abc").as_bytes()).is_empty());
        assert_eq!(parser.flush_pending_escape(), Some(alt));
        assert_eq!(parser.push(b""), vec![InputEvent::Text("abc".into())]);
    }
}

#[test]
fn invalid_string_payload_falls_back_without_swallowing_keys() {
    for tail in ["\x03", "\r", "\t", "\x1b[A"] {
        assert_eq!(
            parse_chunks(&[&format!("\x1b_P{tail}")]),
            [
                vec![
                    InputEvent::Text("\x1b_".into()),
                    InputEvent::Text("P".into())
                ],
                parse_chunks(&[tail]),
            ]
            .concat()
        );
    }
    assert_eq!(
        parse_chunks(&["\x1b_P\u{4e2d}"]),
        vec![
            InputEvent::Text("\x1b_".into()),
            InputEvent::Text("P\u{4e2d}".into())
        ]
    );
}

#[test]
fn terminal_strings_in_paste_remain_raw() {
    let payload = "\x1b_Gi=31;OK\x1b\\\x1b]52;c;YWJj\x07";
    assert_eq!(
        parse_chunks(&[&format!("\x1b[200~{payload}\x1b[201~")]),
        vec![InputEvent::Paste(payload.into())]
    );
}
