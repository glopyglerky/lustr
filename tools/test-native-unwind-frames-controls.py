#!/usr/bin/env python3
"""Negative controls for the DWARF frame-walk evidence evaluator."""
import importlib.util
from pathlib import Path
import unittest


spec = importlib.util.spec_from_file_location("unwind_frames", Path(__file__).with_name("test-native-unwind-frames.py"))
gate = importlib.util.module_from_spec(spec)
spec.loader.exec_module(gate)


# Minimal excerpts of the documented GDB trace format. The address walk alone
# succeeds even when a prologue unwinder supplies it; these controls ensure the
# evaluator rejects that fallback and an attempted-but-rejected DWARF sniffer.
TRACE = b'''LUSTR_UNWIND_TRACE_BEGIN
  [frame] frame_unwind_find_by_frame: this_frame=0
  [frame] frame_unwind_try_unwinder: trying unwinder "dwarf2"
  [frame] frame_unwind_try_unwinder: yes
  [frame] get_prev_frame_always_1: -> {level=0,type=NORMAL_FRAME,unwinder="dwarf2",pc=0x1004,id=<computing>}
    [frame] frame_unwind_find_by_frame: this_frame=1
    [frame] frame_unwind_try_unwinder: trying unwinder "dwarf2"
    [frame] frame_unwind_try_unwinder: yes
    [frame] get_prev_frame_always_1: -> {level=1,type=NORMAL_FRAME,unwinder="dwarf2",pc=0x2030,id=<computing>}
LUSTR_UNWIND_TRACE_END
'''


def observation(trace=TRACE):
    return {"entry": 0x1000, "stop": 0x1004, "error": None,
            "frames": [{"level": level, "pc": pc} for level, pc in enumerate((0x1004, 0x2030, 0x3040))],
            "unwinder_trace": gate.selected_unwinders(trace)}


def accepted(value):
    return gate.frame_sequence(value, [{"value": start, "size": 0x80} for start in (0x1000, 0x2000, 0x3000)], 4)


class EvidenceControls(unittest.TestCase):
    def test_valid_dwarf_walk_passes(self):
        self.assertTrue(accepted(observation()))

    def test_correct_addresses_with_frame_pointer_fallback_fail(self):
        trace = TRACE.replace(b'"dwarf2"', b'"amd64 prologue"')
        self.assertFalse(accepted(observation(trace)))

    def test_rejected_dwarf_attempt_does_not_prove_selection(self):
        trace = TRACE.replace(b'    [frame] frame_unwind_try_unwinder: yes',
            b'    [frame] frame_unwind_try_unwinder: no\n'
            b'    [frame] frame_unwind_try_unwinder: trying unwinder "amd64 prologue"\n'
            b'    [frame] frame_unwind_try_unwinder: yes')
        self.assertFalse(accepted(observation(trace)))

    def test_selected_dwarf_at_different_pc_fails(self):
        self.assertFalse(accepted(observation(TRACE.replace(b'pc=0x2030', b'pc=0x2040'))))

    def test_wrong_rust_caller_fails(self):
        value = observation()
        value["frames"][2]["pc"] = 0x4000
        self.assertFalse(accepted(value))

    def test_frame_walk_error_fails(self):
        value = observation()
        value["error"] = "Cannot access memory"
        self.assertFalse(accepted(value))

    def test_missing_trace_is_an_error_not_a_negative_control_pass(self):
        with self.assertRaises(AssertionError):
            gate.selected_unwinders(b"ptrace: Operation not permitted\n")

    def test_truncated_trace_is_an_error(self):
        with self.assertRaises(AssertionError):
            gate.selected_unwinders(TRACE.replace(b"LUSTR_UNWIND_TRACE_END\n", b""))

    def test_yes_without_matching_attempt_is_an_error(self):
        with self.assertRaises(AssertionError):
            gate.selected_unwinders(TRACE.replace(b'  [frame] frame_unwind_try_unwinder: trying unwinder "dwarf2"\n', b'', 1))

    def test_nested_selection_is_not_attributed_to_parent_frame(self):
        nested = (b'    [frame] frame_unwind_find_by_frame: this_frame=4\n'
                  b'    [frame] frame_unwind_try_unwinder: trying unwinder "amd64 prologue"\n'
                  b'    [frame] frame_unwind_try_unwinder: yes\n')
        trace = TRACE.replace(b'  [frame] frame_unwind_try_unwinder: yes\n',
                              nested + b'  [frame] frame_unwind_try_unwinder: yes\n', 1)
        self.assertTrue(accepted(observation(trace)))

    def test_wrong_frame_levels_fail(self):
        value = observation()
        value["frames"][1]["level"] = 2
        self.assertFalse(accepted(value))


if __name__ == "__main__":
    unittest.main()
