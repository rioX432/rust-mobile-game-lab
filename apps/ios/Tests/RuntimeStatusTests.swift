import Foundation

@main
enum RuntimeStatusTests {
    static func main() {
        let probe = #"{"game":null,"last_action_error":null,"touches":3}"#
        precondition(RuntimeStatus.readable(probe).hasPrefix("Probe\n"))
        precondition(RuntimeStatus.readable(probe).hasSuffix(probe))
        let selecting = #"{"game":{"round_id":1,"phase":"Selecting","result":null},"last_action_error":null}"#
        let active = RuntimeStatus.readable(selecting)
        precondition(active.contains("Race round 1 / Selecting"))
        precondition(active.contains("Result (lanes 0..2): pending"))
        let result = #"{"game":{"round_id":2,"phase":"Result","result":{"lane_order":[0,2,1],"counts":[1,0,0],"winners":[2]}},"last_action_error":"WrongRound","last_error":null}"#
        let finished = RuntimeStatus.readable(result)
        precondition(finished.contains("Race round 2 / Result"))
        precondition(finished.contains("order=[0, 2, 1] counts=[1, 0, 0] winners=[2]"))
        precondition(finished.contains("Action error: WrongRound"))
        precondition(RuntimeStatus.readable("invalid JSON") == "invalid JSON")
        print("RuntimeStatus: Probe, selecting, result, action error and invalid JSON checks passed")
    }
}
