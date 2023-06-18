import { BsClouds, BsFillArrowUpLeftSquareFill } from "react-icons/bs";
import { Container, Row, Col, Button } from "react-bootstrap";
import { ButtonLink } from "./ButtonLink";

function ProjectHeader({ project, projectName }) {
  const renderViewButton = () => {
    if (project.prodDeploymentId) {
      return (
        <Button
          variant="primary"
          size="sm"
          className="ms-2"
          target="_blank"
          href={project.prodDeployment.urlsList[0]}
        >
          <BsClouds size={16} className="icon" />
          View
        </Button>
      );
    }
    return null;
  };

  return (
    <header id="project-header">
      <Container>
        <Row>
          <Col md={6} sm={6} xs={5} id="project-header-left">
            <h2>{projectName}</h2>
            <p>
              {project.prodDeploymentId ? "In Production" : "No Deployment"}
            </p>
          </Col>
          <Col id="project-header-right">
            <ButtonLink variant="secondary" size="sm" to="/projects">
              <BsFillArrowUpLeftSquareFill size={16} className="icon" />
              Projects
            </ButtonLink>
            {renderViewButton()}
          </Col>
        </Row>
      </Container>
    </header>
  );
}

export default ProjectHeader;
