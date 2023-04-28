import { Container, Nav, Navbar, NavDropdown, Image } from "react-bootstrap";
import { userAuthContext } from "./AuthContext";
import { BsCheckCircle, BsPlus } from "react-icons/bs";

function DashboardNavbar() {
  const user = userAuthContext().user;
  const avatarLogo = (
    <span>
      <Image
        className="dashboard-navbar-avatar"
        src={user.avatarUrl}
        rounded
        width={26}
        height={26}
      />
      <span>{user.displayName}</span>
    </span>
  );
  console.log("----user", user);
  return (
    <Navbar bg="light" expand="lg" className="dashboard-navbar">
      <Container>
        <Navbar.Brand href="/">Moni-Web</Navbar.Brand>
        <Navbar.Toggle aria-controls="dashboard-navbar-nav" />
        <Navbar.Collapse id="dashboard-navbar-nav">
          <Nav className="me-auto">
            <NavDropdown title={avatarLogo} id="dashboard-nav-dropdown">
              <NavDropdown.Item id="current-account">
                <BsCheckCircle size={16} />
                <span className="account-name">{user.displayName}</span>
              </NavDropdown.Item>
              <NavDropdown.Divider />
              <NavDropdown.Item href="/org/new" id="create-org-nav" disabled>
                <BsPlus size={16} />
                <span className="create-org">New Organization</span>
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
              title={
                <Image
                  className="dashboard-navbar-avatar"
                  src={user.avatarUrl}
                  rounded
                  width={26}
                  height={26}
                />
              }
              align="end"
              id="dashboard-profile-dropdown"
            >
              <div className="profile-avatar">
                <Image src={user.avatarUrl} roundedCircle width={80} height={80} />
                <p><h5>{user.displayName}</h5></p>
              </div>
              <NavDropdown.Divider />
              <NavDropdown.Item href="/dashboard">Projects</NavDropdown.Item>
              <NavDropdown.Item href="/access-tokens">
                Access Tokens
              </NavDropdown.Item>
              <NavDropdown.Divider />
              <NavDropdown.Item href="/signout">Sign Out</NavDropdown.Item>
            </NavDropdown>
          </Nav>
        </Navbar.Collapse>
      </Container>
    </Navbar>
  );
}

export default DashboardNavbar;
