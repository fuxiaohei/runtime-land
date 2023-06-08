import { Container, Nav } from "react-bootstrap";
import { NavbarLink } from "./ButtonLink";

function ProjectTabs({ projectName, activeKey }) {
  return (
    <div id="project-tabs-container">
      <Container>
        <Nav activeKey={"link-" + activeKey}>
          <Nav.Item>
            <NavbarLink
              eventKey="link-overview"
              to={"/projects/" + projectName}
            >
              Overview
            </NavbarLink>
          </Nav.Item>
          <Nav.Item>
            <NavbarLink
              eventKey="link-deployments"
              to={"/projects/" + projectName + "/deployments"}
            >
              Deployments
            </NavbarLink>
          </Nav.Item>
          <Nav.Item>
            <NavbarLink
              eventKey="link-settings"
              to={"/projects/" + projectName + "/settings"}
            >
              Settings
            </NavbarLink>
          </Nav.Item>
          <Nav.Item>
            <Nav.Link eventKey="disabled" disabled>
              Logs
            </Nav.Link>
          </Nav.Item>
          <Nav.Item>
            <Nav.Link eventKey="disabled" disabled>
              Analytics
            </Nav.Link>
          </Nav.Item>
        </Nav>
      </Container>
    </div>
  );
}

export default ProjectTabs;
