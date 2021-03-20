pub fn assert_eq_ts(s: impl quote::ToTokens, ts: proc_macro2::TokenStream) {
    let ts0 = s.to_token_stream().to_string();
    let ts1 = ts.to_string();
    assert_eq!(ts0, ts1);
}
