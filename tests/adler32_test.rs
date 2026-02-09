use libdeflate::adler32::adler32_generic;

const MAX_CHUNK_LEN: usize = 5552;

#[test]
fn test_adler32_large_chunks() {
    let data_exact = vec![b'A'; MAX_CHUNK_LEN];
    assert_eq!(
        adler32_generic(1, &data_exact),
        2735505916,
        "Failed at exact chunk size"
    );

    let data_plus_one = vec![b'A'; MAX_CHUNK_LEN + 1];
    assert_eq!(
        adler32_generic(1, &data_plus_one),
        626557501,
        "Failed at chunk size + 1"
    );

    let data_large = vec![b'A'; 6000];
    assert_eq!(
        adler32_generic(1, &data_large),
        4027970492,
        "Failed at large size"
    );
}
