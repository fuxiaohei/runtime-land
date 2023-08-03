import { Col, Row, Container, OverlayTrigger, Tooltip } from "react-bootstrap";
import { BiLinkExternal } from "react-icons/bi";
import { Link } from "react-router-dom";
import { ButtonLink } from "../layouts/Links";
import ReactTimeAgo from "react-time-ago";

function ProjectPendingButton({ name }) {
  return (
    <div className="project-btn">
      <ButtonLink
        size="sm"
        to={"/projects/" + name + "/overview"}
        variant="secondary"
      >
        Pending
      </ButtonLink>
    </div>
  );
}

function ProjectRunningButton({ name }) {
  return (
    <div className="project-btn">
      <ButtonLink
        size="sm"
        to={"/projects/" + name + "/overview"}
        variant="success"
      >
        Running
      </ButtonLink>
    </div>
  );
}

function ProjectDevelopmentButton({ name }) {
  return (
    <div className="project-btn">
      <ButtonLink
        size="sm"
        to={"/projects/" + name + "/overview"}
        variant="warning"
      >
        Developing
      </ButtonLink>
    </div>
  );
}

function ProjectsListRow({ data }) {
  const { project, deployments_count, prod_deployment } = data;

  const renderButton = () => {
    if (project.status === "pending") {
      return <ProjectPendingButton name={project.name} />;
    }
    if (prod_deployment) {
      return <ProjectRunningButton name={project.name} />;
    }
    return <ProjectDevelopmentButton name={project.name} />;
  };

  return (
    <Col md={6} className="mb-2 p-2">
      <div className="border project-row rounded-3 p-3 d-flex justify-content-between ">
        <div className="project-metadata pe-1">
          <h5 className="name fw-bolder">
            <Link
              to={"/projects/" + project.name + "/overview"}
              className="text-dark text-decoration-none"
            >
              {project.name}
            </Link>
          </h5>
          <div className="project-metadata-items py-2 text-secondary">
            <div>
              <span className="border-end pe-2 language text-capitalize">
                {project.language}
              </span>
              {deployments_count > 0 ? (
                <span className="border-end px-2 deployment">
                  {deployments_count} Deploys
                </span>
              ) : null}
              <span className="ps-2 updated">
                <ReactTimeAgo date={project.updated_at * 1000} locale="en-US" />
              </span>
            </div>
            {prod_deployment ? (
              <OverlayTrigger
                overlay={<Tooltip>{prod_deployment.prod_url}</Tooltip>}
              >
                <div className="project-link pt-3 text-truncate">
                  <span className="border-end link">
                    <BiLinkExternal className="me-2" />
                    <a
                      href={prod_deployment.prod_url}
                      target="_blank"
                      className="text-secondary"
                    >
                      {new URL(prod_deployment.prod_url).hostname}
                    </a>
                  </span>
                </div>
              </OverlayTrigger>
            ) : null}
          </div>
        </div>
        {renderButton()}
      </div>
    </Col>
  );
}

function ProjectsList({ projects }) {
  const renderProjects = () => {
    return projects.map((project) => {
      return <ProjectsListRow key={project.project.uuid} data={project} />;
    });
  };
  return (
    <Container className="my-3">
      <Row>{renderProjects()}</Row>
    </Container>
  );
}

export default ProjectsList;
