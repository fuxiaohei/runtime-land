import { Container, Nav, Navbar } from "react-bootstrap";

function LoginNavbar() {
  return (
    <Navbar className="login-navbar" expand="lg">
      <Container>
        <Navbar.Brand href="/">
          <img
            alt=""
            src="/public/logo-v2-small.svg"
            width="30"
            height="30"
            className="d-inline-block align-top me-2"
          />{" "}
          Runtime.land
        </Navbar.Brand>
        <Navbar.Toggle />
        <Navbar.Collapse className="justify-content-end">
          <Nav>
            <Nav.Link href={window.DOCS_ADDRESS} target="_blank">
              Docs
            </Nav.Link>
            <Nav.Link href={window.FEEDBACK_ADDRESS} target="_blank">
              Feedback
            </Nav.Link>
          </Nav>
        </Navbar.Collapse>
      </Container>
    </Navbar>
  );
}

export default LoginNavbar;
