import { Container, Spinner } from "react-bootstrap";

function LoadingPage() {
  return (
    <Container className="text-center mt-5">
      <div className="mb-4">
        <img width={60} src="/public/logo-v2-small.svg" />
      </div>
      <div>
        <Spinner className="ms-4" animation="border" role="status">
          <span className="visually-hidden">Loading...</span>
        </Spinner>
        <span className="ps-4 fs-3">Loading...</span>
      </div>
    </Container>
  );
}

export default LoadingPage;
