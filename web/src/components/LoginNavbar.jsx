import { Container, Nav, Navbar, NavDropdown } from "react-bootstrap";

function LoginNavbar() {
  return (
    <Navbar className="login-navbar" expand="lg">
      <Container>
        <Navbar.Brand href="/">Runtime.land</Navbar.Brand>
        <Navbar.Toggle />
        <Navbar.Collapse className="justify-content-end">
          <Nav>
            <Nav.Link href="#docs">Docs</Nav.Link>
            <Nav.Link href="#contacts">Contacts</Nav.Link>
          </Nav>
        </Navbar.Collapse>
      </Container>
    </Navbar>
  );
}

export default LoginNavbar;
