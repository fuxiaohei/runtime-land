import { Button, Nav } from "react-bootstrap";
import { BiLinkExternal } from "react-icons/bi";
import { NavbarLink } from "../layouts/Links";

function ProjectHeader({ activeKey, project }) {
  const renderButton = () => {
    if (project.status === "pending") {
      return <Button variant="secondary">Pending</Button>;
    }
    if (project.prod_deployment) {
      return (
        <Button variant="success">
          <BiLinkExternal className="me-2" />
          <a
            href={project.prod_url}
            className="text-white text-decoration-none"
            target="_blank"
          >
            View
          </a>
        </Button>
      );
    }
    return <Button variant="warning">Developing</Button>;
  };

  return (
    <div className="overview-header-container">
      <div className="overview-header d-flex justify-content-between mt-4">
        <h3>{project.name}</h3>
        {renderButton()}
      </div>
      <Nav className="my-3" variant="underline" defaultActiveKey={activeKey}>
        <Nav.Item>
          <NavbarLink
            className="me-3"
            eventKey="overview"
            to={"/projects/" + project.name + "/overview"}
          >
            Overview
          </NavbarLink>
        </Nav.Item>
        <Nav.Item>
          <Nav.Link className="mx-3" eventKey="traffic" disabled>
            Traffic
          </Nav.Link>
        </Nav.Item>
        <Nav.Item>
          <NavbarLink
            className="mx-3"
            eventKey="setting"
            to={"/projects/" + project.name + "/setting"}
          >
            Settings
          </NavbarLink>
        </Nav.Item>
      </Nav>
    </div>
  );
}

export default ProjectHeader;
