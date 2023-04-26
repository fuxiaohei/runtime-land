import LoginNavbar from "./LoginNavbar";
import { Button, Container } from "react-bootstrap";

function NotFoundPage() {
  return (
    <div>
      <LoginNavbar />
      <Container id="notfound-container">
        <h1>Page Not Found</h1>
        <p>Sorry, but the page you were trying to view does not exist.</p>
        <Button href="/" variant="primary">
          Go Home
        </Button>
      </Container>
    </div>
  );
}

export default NotFoundPage;
