use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_attribute]
pub fn http_main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = syn::parse_macro_input!(item as syn::ItemFn);
    let func_name = func.sig.ident.clone();

    let wit_guest_rs = include_str!("../../../wit/http-handler.rs").to_string();
    let iface: TokenStream = wit_guest_rs.parse().expect("cannot parse http_handler.rs");
    let iface_impl = quote!(
        struct HttpImpl;

        impl http_handler::HttpHandler for HttpImpl {
            fn handle_request(req: http_handler::Request) -> http_handler::Response {
                #func

                let http_req: Request = match req.try_into() {
                    Ok(r) => r,
                    Err(e) => {
                        return http_handler::Response {
                            status: 500,
                            headers: vec![],
                            body: Some(format!("Request Convert Error: {:?}", e).as_bytes().to_vec()),
                        }
                    }
                };
                let http_resp = #func_name(http_req);
                match http_resp.try_into() {
                    Ok(r) => r,
                    Err(e) => http_handler::Response {
                        status: 500,
                        headers: vec![],
                        body: Some(format!("Response Convert Error: {:?}", e).as_bytes().to_vec()),
                    },
                }
            }
        }

        impl TryFrom<http_handler::Request> for http::Request<bytes::Bytes> {
            type Error = anyhow::Error;

            fn try_from(leaf_req: http_handler::Request) -> Result<Self, Self::Error> {
                use std::str::FromStr;

                let mut http_req = http::Request::builder()
                    .method(http::Method::from_str(leaf_req.method.as_str())?)
                    .uri(&leaf_req.uri);

                for (key, value) in leaf_req.headers {
                    http_req = http_req.header(key, value);
                }

                let body = match leaf_req.body {
                    Some(b) => b.to_vec(),
                    None => Vec::new(),
                };

                let body = bytes::Bytes::from(body);

                Ok(http_req.body(body)?)
            }
        }

        impl TryFrom<http::Response<bytes::Bytes>> for http_handler::Response {
            type Error = anyhow::Error;

            fn try_from(http_res: http::Response<bytes::Bytes>) -> Result<Self, Self::Error> {
                let status = http_res.status().as_u16();
                let mut headers: Vec<(String, String)> = vec![];
                for (key, value) in http_res.headers() {
                    headers.push((key.to_string(), value.to_str()?.to_string()));
                }
                let body = http_res.body();
                Ok(http_handler::Response {
                    status,
                    headers,
                    body:Some(body.to_vec()),
                })
            }
        }

        export_http_handler!(HttpImpl);

    );

    let value = format!("{iface}\n{iface_impl}");
    value.parse().unwrap()
}
