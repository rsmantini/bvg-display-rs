# bvg-display-rs

A departure display for Berlin public transport (BVG), written in Rust.
It fetches live departure data from the
[BVG Open API v6](https://v6.bvg.transport.rest/api.html) and renders it
in a native UI built with [iced](https://iced.rs/).

Two UI themes are available:

| Modern | Retro |
|--------|-------|
| ![](assets/images/modern-ui.png) | ![](assets/images/retro-ui.png) |

## Finding stop IDs

To find the stop ID of a station for the `bvg_stops` config, run:

```
curl 'https://v6.bvg.transport.rest/locations?poi=false&addresses=false&query=alexander' -s | jq
```

and grab the `id` of the desired station from the response. For example:

```json
{
    "type": "stop",
    "id": "900100003",
    "name": "S+U Alexanderplatz Bhf (Berlin)",
    "location": {
      "type": "location",
      "id": "900100003",
      "latitude": 52.521508,
      "longitude": 13.411267
    },
    "products": {
      "suburban": true,
      "subway": true,
      "tram": true,
      "bus": true,
      "ferry": false,
      "express": false,
      "regional": false
    },
    ...
}
```