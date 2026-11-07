// HTTP response types

#[derive(Debug)]
pub struct HttpResponse<T> {
    pub status_code: u16,
    pub body: T,
}
