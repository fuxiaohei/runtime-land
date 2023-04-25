import Container from "react-bootstrap/Container";
import Nav from "react-bootstrap/Nav";
import Navbar from "react-bootstrap/Navbar";
import NavDropdown from "react-bootstrap/NavDropdown";

function DashboardNavbar() {
  return (
    <Navbar bg="light" expand="lg" className="dashboard-navbar">
      <Container>
        <Navbar.Brand href="#home">Moni-Web</Navbar.Brand>
        <Navbar.Toggle aria-controls="dashboard-navbar-nav" />
        <Navbar.Collapse id="dashboard-navbar-nav">
          <Nav className="me-auto">
            <NavDropdown title="FuXiaoHei" id="dashboard-nav-dropdown">
              <NavDropdown.Item href="#action/3.1">FuXiaoHei</NavDropdown.Item>
              <NavDropdown.Divider />
              <NavDropdown.Item href="#action/3.2">
                Create Organization
              </NavDropdown.Item>
              <NavDropdown.Item href="#action/3.4">BBB Inc.</NavDropdown.Item>
            </NavDropdown>
          </Nav>
        </Navbar.Collapse>
        <Navbar.Collapse className="justify-content-end">
          <Nav>
            <Nav.Link href="#home">Docs</Nav.Link>
            <Nav.Link href="#link">Feedback</Nav.Link>
            <NavDropdown
              title="profile"
              align="end"
              id="dashboard-profile-dropdown"
            >
              Signed in as: <a href="#login">Mark Otto</a>
            </NavDropdown>
          </Nav>
        </Navbar.Collapse>
      </Container>
    </Navbar>
  );
}

export default DashboardNavbar;
