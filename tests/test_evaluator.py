import evalrspy
import json

def test_evaluator():
    payload = {
        "script": "a+9",
        "variables": {
            "a": 9
        }
    }
    result = evalrspy.evaluate(json.dumps(payload))

    expected = {
        "status": "ok",
        "result": 18
    }

    assert json.loads(result) == expected
