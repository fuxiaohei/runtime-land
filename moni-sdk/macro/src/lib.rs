use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_attribute]
pub fn http_main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = syn::parse_macro_input!(item as syn::ItemFn);
    let func_name = func.sig.ident.clone();

    let wit_guest_rs = include_str!("../../../wit-v2/http_handler.rs").to_string();
    let iface: TokenStream = wit_guest_rs.parse().expect("cannot parse http_handler.rs");

    let iface_impl = quote!(

        use exports::moni::http::http_incoming;
        
        struct HttpImpl;

        impl TryFrom<http_incoming::Request> for Request {
            type Error = anyhow::Error;

            fn try_from(wasm_req: http_incoming::Request) -> Result<Self, Self::Error> {
                use std::str::FromStr;

                let mut http_req = http::Request::builder()
                    .method(http::Method::from_str(wasm_req.method.as_str())?)
                    .uri(&wasm_req.uri);

                for (key, value) in wasm_req.headers {
                    http_req = http_req.header(key, value);
                }
                // 1 is the request body handle, which is defined in wasi host functions
                let body = Body::new(wasm_req.body.unwrap_or(1));
                Ok(http_req.body(body)?)
            }
        }

        impl TryFrom<Response> for http_incoming::Response {
            type Error = anyhow::Error;

            fn try_from(http_res: Response) -> Result<Self, Self::Error> {
                let status = http_res.status().as_u16();
                let mut headers: Vec<(String, String)> = vec![];
                for (key, value) in http_res.headers() {
                    headers.push((key.to_string(), value.to_str()?.to_string()));
                }
                let body = http_res.body();
                Ok(http_incoming::Response {
                    status,
                    headers,
                    body: Some(body.body_handle()),
                })
            }
        }

        impl http_incoming::HttpIncoming for HttpImpl {
            fn handle_request(req: http_incoming::Request) -> http_incoming::Response {
                #func

                // convert wasm_request to sdk_request
                let sdk_request: Request = req.try_into().unwrap();
                let sdk_response = #func_name(sdk_request);

                let sdk_response_body_handle = sdk_response.body().body_handle();
                // convert sdk_response to wasm_response
                match sdk_response.try_into() {
                    Ok(r) => r,
                    Err(_e) => http_incoming::Response {
                        status: 500,
                        headers: vec![],
                        body: Some(sdk_response_body_handle),
                    },
                }
            }
        }

        export_http_handler!(HttpImpl);

    );
    let value = format!("{iface}\n{iface_impl}");
    value.parse().unwrap()
}
