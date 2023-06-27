import LoginNavbar from "../components/LoginNavbar";
import { Button, Container } from "react-bootstrap";
import { useNavigate } from "react-router-dom";
import { Helmet } from "react-helmet-async";

function NotFoundPage() {
  const navigate = useNavigate();
  return (
    <div>
      <Helmet>
        <title>Page Not Found | Runtime.land</title>
      </Helmet>
      <LoginNavbar />
      <Container id="notfound-container">
        <h1>Page Not Found</h1>
        <p>Sorry, but the page you were trying to view does not exist.</p>
        <Button
          onClick={() => {
            navigate(-1);
          }}
          variant="primary"
        >
          Go Home
        </Button>
      </Container>
    </div>
  );
}

export default NotFoundPage;
