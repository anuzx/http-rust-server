// HTTP response types

pub struct HttpResponse<T> {
    pub status_code : u16, //http status code are non-negative int
    pub body: T,
}