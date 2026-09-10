
mod routes;
mod workers;
// mod error;

pub use routes::*;

/*
 * uploads/
 ├── staging/
 │   └── <UUID-A>/
 │       ├── video.part
 │       └── manifest.json
 │
 ├── processing/
 │   ├── <UUID-B>/
 │   │   ├── video.mp4
 │   │   ├── audio.part
 │   │   └── manifest.json
 │   │
 │   └── <UUID-B>/
 │       ├── video.mp4
 │       ├── audio.wav
 │       └── manifest.json
 │
 ├── failed/
 │   └── <UUID-A>/
 │       ├── video.mp4
 │       ├── audio.part
 │       └── manifest.json
 │
 └── final/
     └── <UUID-A>/
         ├── video.mp4
         ├── audio.wav
         └── manifest.json
 */

 /*
  * {
    "id": "0198f9c4-5555-7aaa-8bbb-555555555555",
    "state": "final",
  
    "source": {
    // The filename uploaded by user could
    // be dangerous, so unless needed later,
    // we dont have to use it
      // "filename": "lecture-01.mp4",
      "path": "video.mp4",
      "size_bytes": 183742912,
      "container": "mp4",
      "media_type": "video"
    },
  
    "artifacts": [
      {
        "media_type": "video",
        "codec": "h264",
        "width": 1920,
        "height": 1080,
        "fps": {
          "numerator": 30,
          "denominator": 1
        },
        "duration_ms": 4215000
      },
      {
        "media_type": "audio",
        "codec": "aac",
        "sample_rate": 48000,
        "channels": 2,
        "duration_ms": 4215000
      }
    ],
  
    "created_at": "2026-08-29T14:30:00Z",
    "updated_at": "2026-08-29T14:35:42Z",
  
    "error": null
  }
  */