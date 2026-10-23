import Foundation

enum RuntimeStatus {
    static func readable(_ json: String) -> String {
        guard let data = json.data(using: .utf8),
              let root = try? JSONSerialization.jsonObject(with: data) as? [String: Any] else { return json }
        let summary: String
        if let game = root["game"] as? [String: Any] {
            let round = game["round_id"] as? NSNumber ?? 0
            let phase = game["phase"] as? String ?? "unknown"
            let result: String
            if let outcome = game["result"] as? [String: Any] {
                let order = outcome["lane_order"] as? [Int] ?? []
                let counts = outcome["counts"] as? [Int] ?? []
                let winners = outcome["winners"] as? [UInt64] ?? []
                result = "order=\(order) counts=\(counts) winners=\(winners)"
            } else {
                result = "pending"
            }
            summary = "Race round \(round) / \(phase)\nResult (lanes 0..2): \(result)"
        } else {
            summary = "Probe"
        }
        return "\(summary)\nAction error: \(root["last_action_error"] ?? "unknown")\n\(json)"
    }
}
