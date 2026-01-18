use std::{
    io::{BufRead, BufReader, Write},
    net::TcpStream,
};

const WEBPAGE: &str = "<script>\r
    const win=window.open(\"https://tiktok.com\", \"_blank\", \"popup\");\r
    setInterval(async ()=> {\r
        try {\r
            const res=await fetch(\"/alive\");\r
            await res.text();\r
            if (!res.ok) {\r
                throw new Exception(\"Response was not ok\")\r
            }\r
        }catch(_) {\r
            win.close();\r
            close();\r
        }\r
    }, 1000)\r
    </script>\n\r";

struct Resp {
    status: u16,
    content: &'static str,
    content_type: &'static str,
}
impl Resp {
    pub const fn new(status: u16, content_type: &'static str, content: &'static str) -> Self {
        Self {
            status,
            content,
            content_type,
        }
    }

    pub fn write(&self, mut stream: impl Write) -> std::io::Result<()> {
        let status = match self.status {
            200 => "200 Ok",
            400 => "400 BadRequest",
            404 => "404 NotFound",
            _ => "500 InternalServerError",
        };
        let headers = format_args!("Content-Type: {}\r\n", self.content_type);

        let resp = format!("HTTP/1.1 {}\r\n{}\r\n{}", status, headers, self.content);
        stream.write_all(resp.as_bytes())?;
        Ok(())
    }
}

pub struct TinyHttp {
    line_buf: String,
}
impl TinyHttp {
    pub fn new() -> Self {
        Self {
            line_buf: String::new(),
        }
    }

    fn handler(&self, method: &str, path: &str) -> Resp {
        match (method, path) {
            ("GET", "/") => Resp::new(200, "text/html", WEBPAGE),
            ("GET", "/alive") => Resp::new(200, "text/plain", "ok"),
            _ => {
                println!("got {method} {path}");
                Resp::new(404, "text/plain", "notfound")
            }
        }
    }

    fn parse_request_line(line: &str) -> Option<(&str, &str)> {
        let mut line = line.split(" ");
        let method = line.next()?;
        let path = line.next()?;
        Some((method, path))
    }

    pub fn handle_req(&mut self, stream: TcpStream) -> std::io::Result<()> {
        let mut buf_reader = BufReader::new(stream.try_clone()?);
        buf_reader.read_line(&mut self.line_buf)?;

        let resp = match Self::parse_request_line(&self.line_buf) {
            Some((method, path)) => self.handler(method, path),
            None => Resp::new(400, "text/plain", "http error"),
        };

        resp.write(stream)?;
        Ok(())
    }
}
