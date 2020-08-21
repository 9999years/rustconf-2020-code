#! /usr/bin/env python3.8

import json

import requests

with open("openweather_api.json") as f:
    api_key_obj = json.load(f)
    api_key = api_key_obj["api_key"]
    res = requests.get(
        "https://api.openweathermap.org/data/2.5/weather",
        params={"q": "Waltham,MA,US", "appid": api_key},
    )
    print(res.text)
