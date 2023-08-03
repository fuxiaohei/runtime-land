import { Container, Navbar, Nav, NavDropdown } from "react-bootstrap";
import { Link } from "react-router-dom";
import { NavbarLink, NavDropdownLink } from "./Links";
import { useClerk } from "@clerk/clerk-react";

function MainLayout({ children }) {
  const { signOut } = useClerk();

  const handleSignOut = (event) => {
    console.log("sign out");
    event.preventDefault();
    signOut();
    console.log("call sign out");
  };

  return (
    <main>
      <header>
        <Navbar expand="lg" className="bg-body-tertiary py-2 border-bottom">
          <Container className="d-flex justify-content-between">
            <Navbar.Brand as="span">
              <img src="/public/logo-v2-small.svg" width={40} />
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
                  src="https://avatars.githubusercontent.com/u/2142787?v=4"
                  width={36}
                />
                <NavDropdown
                  as="span"
                  id="nav-account-dropdown"
                  title="FuXiaohei"
                >
                  <NavDropdownLink to="/account">
                    Account Settings
                  </NavDropdownLink>
                  <NavDropdown.Divider />
                  <NavDropdown.Item onClick={handleSignOut}>
                    Log out
                  </NavDropdown.Item>
                </NavDropdown>
              </Navbar.Text>
            </Navbar.Collapse>
          </Container>
        </Navbar>
      </header>
      {children}
      <footer className="text-center text-secondary border-top mt-5 py-4">
        @2023 Runtime.land
      </footer>
    </main>
  );
}

export default MainLayout;
