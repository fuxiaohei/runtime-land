import LoginNavbar from "../components/LoginNavbar";
import { Container } from "react-bootstrap";

function LoginLoadingPage() {
  return (
    <div>
      <LoginNavbar />
      <Container id="login-loading-container">
        <h1>Login</h1>
        <p>waiting for login to complete...</p>
      </Container>
    </div>
  );
}

export default LoginLoadingPage;
