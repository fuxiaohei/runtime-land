import { Button, Container } from "react-bootstrap";
import { useNavigate } from "react-router-dom";
import { ButtonLink } from "../layouts/Links";

function NotFoundPage() {
  const nav = useNavigate();
  return (
    <Container id="notfound-page" className="text-center mt-5">
      <div className="mb-4">
        <img width={80} src="/public/logo-v2-small.svg" />
      </div>
      <div>
        <h2>Page Not Found</h2>
      </div>
      <div className="mt-4">
        <ButtonLink to="/" className="mx-3" variant="primary">
          Home
        </ButtonLink>
        <Button onClick={() => nav(-1)} className="mx-3" variant="secondary">
          Back
        </Button>
      </div>
    </Container>
  );
}

export default NotFoundPage;
