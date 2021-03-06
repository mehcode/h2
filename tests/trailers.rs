#[macro_use]
extern crate log;

extern crate h2_test_support;
use h2_test_support::prelude::*;

#[test]
fn recv_trailers_only() {
    let _ = env_logger::init();

    let mock = mock_io::Builder::new()
        .handshake()
        // Write GET /
        .write(&[
            0, 0, 0x10, 1, 5, 0, 0, 0, 1, 0x82, 0x87, 0x41, 0x8B, 0x9D, 0x29,
                0xAC, 0x4B, 0x8F, 0xA8, 0xE9, 0x19, 0x97, 0x21, 0xE9, 0x84,
        ])
        .write(frames::SETTINGS_ACK)
        // Read response
        .read(&[
              0, 0, 1, 1, 4, 0, 0, 0, 1, 0x88, 0, 0, 9, 1, 5, 0, 0, 0, 1,
              0x40, 0x84, 0x42, 0x46, 0x9B, 0x51, 0x82, 0x3F, 0x5F,
        ])
        .build();

    let mut h2 = Client::handshake(mock).wait().unwrap();

    // Send the request
    let request = Request::builder()
        .uri("https://http2.akamai.com/")
        .body(())
        .unwrap();

    info!("sending request");
    let mut stream = h2.request(request, true).unwrap();

    let response = h2.run(poll_fn(|| stream.poll_response())).unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let (_, mut body) = response.into_parts();

    // Make sure there is no body
    let chunk = h2.run(poll_fn(|| body.poll())).unwrap();
    assert!(chunk.is_none());

    let trailers = h2.run(poll_fn(|| body.poll_trailers())).unwrap().unwrap();
    assert_eq!(1, trailers.len());
    assert_eq!(trailers["status"], "ok");

    h2.wait().unwrap();
}

#[test]
fn send_trailers_immediately() {
    let _ = env_logger::init();

    let mock = mock_io::Builder::new()
        .handshake()
        // Write GET /
        .write(&[
               0, 0, 0x10, 1, 4, 0, 0, 0, 1, 0x82, 0x87, 0x41, 0x8B, 0x9D, 0x29,
               0xAC, 0x4B, 0x8F, 0xA8, 0xE9, 0x19, 0x97, 0x21, 0xE9, 0x84, 0, 0,
               0x0A, 1, 5, 0, 0, 0, 1, 0x40, 0x83, 0xF6, 0x7A, 0x66, 0x84, 0x9C,
               0xB4, 0x50, 0x7F,
        ])
        .write(frames::SETTINGS_ACK)
        // Read response
        .read(&[
               0, 0, 1, 1, 4, 0, 0, 0, 1, 0x88, 0, 0, 0x0B, 0, 1, 0, 0, 0, 1,
               0x68, 0x65, 0x6C, 0x6C, 0x6F, 0x20, 0x77, 0x6F, 0x72, 0x6C, 0x64,
        ])
        .build();

    let mut h2 = Client::handshake(mock).wait().unwrap();

    // Send the request
    let request = Request::builder()
        .uri("https://http2.akamai.com/")
        .body(())
        .unwrap();

    info!("sending request");
    let mut stream = h2.request(request, false).unwrap();

    let mut trailers = HeaderMap::new();
    trailers.insert("zomg", "hello".parse().unwrap());

    stream.send_trailers(trailers).unwrap();

    let response = h2.run(poll_fn(|| stream.poll_response())).unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let (_, mut body) = response.into_parts();

    // There is a data chunk
    let chunk = h2.run(poll_fn(|| body.poll())).unwrap();
    assert!(chunk.is_some());

    let chunk = h2.run(poll_fn(|| body.poll())).unwrap();
    assert!(chunk.is_none());

    let trailers = h2.run(poll_fn(|| body.poll_trailers())).unwrap();
    assert!(trailers.is_none());

    h2.wait().unwrap();
}

#[test]
#[ignore]
fn recv_trailers_without_eos() {
    // This should be a protocol error?
}
