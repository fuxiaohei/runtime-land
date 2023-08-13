import { Container, Navbar, Nav, NavDropdown } from "react-bootstrap";
import { Link } from "react-router-dom";
import { NavbarLink, NavDropdownLink } from "./Links";
import { useClerk } from "@clerk/clerk-react";
import { useAuthContext } from "./AuthContext";
import { Helmet } from "react-helmet-async";
import { version, commitHash, buildDate } from "./verison";

function MainLayout({ title, children }) {
  const { signOut } = useClerk();
  const { user } = useAuthContext();

  const handleSignOut = (event) => {
    console.log("sign out");
    event.preventDefault();
    signOut();
    console.log("call sign out");
  };

  const renderAdminNav = () => {
    if (user.role === "admin") {
      return (
        <>
          <NavDropdownLink to="/admin/stats">Admin</NavDropdownLink>
          <NavDropdown.Divider />
        </>
      );
    }
    return null;
  };

  return (
    <main>
      <Helmet>
        <title>{title}</title>
      </Helmet>
      <header>
        <Navbar expand="lg" className="bg-body-tertiary py-2 border-bottom">
          <Container className="d-flex justify-content-between">
            <Navbar.Brand as="span">
              <img src="/public/logo-v2.svg" width={40} />
              <Link
                className="ms-3 fs-4 align-middle text-dark text-decoration-none"
                to="/"
              >
                Runtime.land
              </Link>
            </Navbar.Brand>
            <Nav>
              <NavbarLink to="/projects" active>
                Projects
              </NavbarLink>
            </Nav>
            <Navbar.Collapse className="justify-content-end">
              <Nav>
                <Nav.Link href="#">Docs</Nav.Link>
                <Nav.Link href="#">Feedback</Nav.Link>
              </Nav>
              <Navbar.Text>
                <img
                  className="rounded ms-3 me-3"
                  src={user.avatar_url}
                  width={36}
                />
                <NavDropdown
                  as="span"
                  id="nav-account-dropdown"
                  title={user.name}
                >
                  <NavDropdownLink to="/account">Account</NavDropdownLink>
                  <NavDropdown.Divider />
                  {renderAdminNav()}
                  <NavDropdown.Item onClick={handleSignOut}>
                    Log out
                  </NavDropdown.Item>
                </NavDropdown>
              </Navbar.Text>
            </Navbar.Collapse>
          </Container>
        </Navbar>
      </header>
      <div className="main-container">{children}</div>
      <footer id="footer" className="text-center text-secondary border-top">
        @2023 Runtime.land | v{version} | {commitHash} | {buildDate}
      </footer>
    </main>
  );
}

export default MainLayout;
