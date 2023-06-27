import LoginNavbar from "../components/LoginNavbar";
import { Container, Spinner } from "react-bootstrap";
import { Helmet } from "react-helmet-async";

function LoginLoadingPage() {
  return (
    <div>
      <Helmet>
        <title>Login | Runtime.land</title>
      </Helmet>
      <LoginNavbar />
      <Container id="login-loading-container">
        <h1>
          Login <Spinner className="mx-4" animation="border" />
        </h1>
        <p> waiting for login to complete...</p>
      </Container>
    </div>
  );
}

export default LoginLoadingPage;
