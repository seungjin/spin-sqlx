use futures::stream::StreamExt;
use futures::SinkExt;
// use spin_executor::bindings::wasi::io::streams;
use spin_sdk::http::{
    Fields, IncomingRequest, OutgoingResponse, ResponseOutparam,
};
use spin_sdk::http_component;

#[derive(Debug, sqlx::FromRow)]
struct Pet {
    age: u32,
    name: String,
    is_finicky: bool,
}

impl std::fmt::Display for Pet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fin_desc = if self.is_finicky { "is" } else { "is not" };
        f.write_fmt(format_args!(
            "{}, aged {}, {} finicky",
            self.name, self.age, fin_desc
        ))
    }
}

#[http_component]
async fn handle_fetch_many(_req: IncomingRequest, resp: ResponseOutparam) {
    let mut headers = spin_sdk::http::Headers::new();
    headers.append(
        &"content-type".to_string(),
        &"text/plain".as_bytes().to_vec(),
    );
    let og = OutgoingResponse::new(headers);
    let mut resp_stm = og.take_body();
    resp.set(og);

    let sqlx_conn = match spin_sqlx::Connection::open_default() {
        Ok(c) => c,
        Err(e) => {
            _ = resp_stm.send(format!("{e:?}").into()).await;
            return;
        }
    };

    let pets = sqlx::query_as::<_, Pet>("SELECT * FROM pets WHERE age < ?")
        .bind(20)
        .fetch(&sqlx_conn);

    let mut resp_lines = pets.map(|pet| match pet {
        Ok(pet) => Ok(format!("{pet}\n").into_bytes()),
        Err(e) => panic!(),
    });

    _ = resp_stm.send_all(&mut resp_lines).await;
}
