import json
import sys

request = json.load(sys.stdin)
print(json.dumps({
    "protocol_version": request["protocol_version"],
    "plugin_id": request["plugin_id"],
    "result_schema_version": 1,
    "results": [{"id": "stew", "value": request["payload"]}],
}))
