import Foundation
@testable import BelunaApp

enum MoiraRuntimeBindingFixtures {
    static func loomSnapshot() -> MoiraLoomSnapshot {
        try! JSONDecoder().decode(MoiraLoomSnapshot.self, from: Data(loomJSON.utf8))
    }

    static let loomJSON = """
    {
      "status": {
        "lifecycle": "ready",
        "resources": [],
        "receiver": {
          "endpoint": "127.0.0.1:4317",
          "wakeState": "listening",
          "dbPath": "/tmp/moira.duckdb",
          "lastBatchAt": null,
          "lastError": null,
          "rawEventCount": 1,
          "wakeCount": 1,
          "tickCount": 1
        },
        "core": {
          "phase": "idle"
        }
      },
      "launchTargets": [
        {
          "target": {
            "kind": "knownLocalBuild",
            "buildId": "dev-core"
          },
          "label": "Dev Core",
          "provenance": "registered",
          "readiness": "ready",
          "executablePath": "/tmp/beluna",
          "workingDir": "/tmp",
          "checksumVerified": false
        }
      ],
      "profiles": [
        {
          "profileId": "default",
          "profilePath": "/tmp/default.jsonc"
        }
      ],
      "runs": [
        {
          "runId": "run-1",
          "firstSeenAt": "2026-05-08T00:00:00Z",
          "lastSeenAt": "2026-05-08T00:00:01Z",
          "eventCount": 1,
          "warningCount": 0,
          "errorCount": 0,
          "latestTick": 3
        }
      ],
      "selectedRunId": "run-1",
      "ticks": [
        {
          "runId": "run-1",
          "tick": 3,
          "traceId": "trace-1",
          "firstSeenAt": "2026-05-08T00:00:00Z",
          "lastSeenAt": "2026-05-08T00:00:01Z",
          "eventCount": 1,
          "warningCount": 0,
          "errorCount": 0,
          "cortexHandled": true
        }
      ],
      "selectedTick": 3,
      "tickDetail": {
        "summary": {
          "runId": "run-1",
          "tick": 3,
          "traceId": "trace-1",
          "firstSeenAt": "2026-05-08T00:00:00Z",
          "lastSeenAt": "2026-05-08T00:00:01Z",
          "eventCount": 1,
          "warningCount": 0,
          "errorCount": 0,
          "cortexHandled": true
        },
        "cortex": [],
        "stem": [],
        "spine": [],
        "raw": [
          {
            "rawEventId": "evt-1",
            "receivedAt": "2026-05-08T00:00:00Z",
            "observedAt": "2026-05-08T00:00:00Z",
            "severityText": "INFO",
            "recordKind": "native_owner",
            "scopeName": "beluna.core.stem.tick",
            "eventName": "started",
            "traceId": "trace-1",
            "spanId": "span-1",
            "traceFlags": 1,
            "target": "stem.tick",
            "family": "stem",
            "subsystem": "stem",
            "runId": "run-1",
            "tick": 3,
            "messageText": "tick started",
            "attributes": {
              "tick": 3
            },
            "body": {
              "summary": "started"
            },
            "resource": {},
            "scope": {}
          }
        ]
      }
    }
    """
}
