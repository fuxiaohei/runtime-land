import { Alert, Container } from "react-bootstrap";

function ErrorPage({ message }) {
  return (
    <Container className="text-center mt-5">
      <div className="mb-4">
        <img width={80} src="/public/logo-v2.svg" />
      </div>
      <div>
        <h2>Oops !</h2>
        <Alert variant="danger" className="mx-auto my-4" id="error-alert">
          <Alert.Heading>Something went wrong</Alert.Heading>
          <p>We are sorry, but something went wrong. Please try again later.</p>
          <hr />
          <p className="mb-0">{message}</p>
        </Alert>
      </div>
    </Container>
  );
}

export default ErrorPage;
