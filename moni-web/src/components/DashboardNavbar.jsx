import { Container, Nav, Navbar, NavDropdown, Image } from "react-bootstrap";
import { userAuthContext } from "./AuthContext";
import { BsCheckCircle, BsPlus } from "react-icons/bs";
import { NavDropdownItemLink, NavbarBrandLink } from "./ButtonLink";

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
  return (
    <Navbar bg="light" expand="lg" className="dashboard-navbar">
      <Container>
        <NavbarBrandLink to="/projects">Moni-Web</NavbarBrandLink>
        <Navbar.Toggle aria-controls="dashboard-navbar-nav" />
        <Navbar.Collapse id="dashboard-navbar-nav">
          <Nav className="me-auto">
            <NavDropdown title={avatarLogo} id="dashboard-nav-dropdown">
              <NavDropdownItemLink id="current-account" to="/projects">
                <BsCheckCircle size={16} />
                <span className="account-name">{user.displayName}</span>
              </NavDropdownItemLink>
              <NavDropdown.Divider />
              <NavDropdownItemLink to="/org/team" id="create-org-nav" disabled>
                <BsPlus size={16} />
                <span className="create-org">New Team</span>
              </NavDropdownItemLink>
              <NavDropdown.Item href="#BBB">BBB Inc.</NavDropdown.Item>
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
                <Image
                  src={user.avatarUrl}
                  roundedCircle
                  width={80}
                  height={80}
                />
                <p>
                  <strong className="profile-displayname">
                    {user.displayName}
                  </strong>
                </p>
              </div>
              <NavDropdown.Divider />
              <NavDropdownItemLink to="/projects">
                Projects
              </NavDropdownItemLink>
              <NavDropdownItemLink to="/settings#access-token">
                Access Tokens
              </NavDropdownItemLink>
              <NavDropdown.Divider />
              <NavDropdownItemLink to="/signout">Sign Out</NavDropdownItemLink>
            </NavDropdown>
          </Nav>
        </Navbar.Collapse>
      </Container>
    </Navbar>
  );
}

export default DashboardNavbar;
