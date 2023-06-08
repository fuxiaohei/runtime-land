import {
  Container,
  Row,
  Col,
  Card,
  ListGroup,
  Dropdown,
} from "react-bootstrap";
import DashboardNavbar from "../components/DashboardNavbar";
import { BsCheck2Circle, BsAppIndicator } from "react-icons/bs";
import { ButtonLink } from "../components/ButtonLink";
import ProjectHeader from "../components/ProjectHeader";
import ProjectTabs from "../components/ProjectTabs";
import { useParams } from "react-router-dom";
import { getProjectOverview } from "../api/project";
import React, { useEffect } from "react";
import ProjectNoDeploymentCard from "../components/ProjectNoDeploymentCard";
import ProjectProdDeploymentCard from "../components/ProjectProdDeploymentCard";
import ProjectDeploymentsListGroup from "../components/ProjectDeploymentsListGroup";

function ProjectPage() {
  const { projectName } = useParams();
  const [projectOverview, setProjectOverview] = React.useState(null);

  const fetchProjectOverview = async () => {
    let project = await getProjectOverview(projectName);
    if (project.error) {
      return;
    }
    setProjectOverview(project);
  };

  useEffect(() => {
    if (!projectOverview) {
      fetchProjectOverview();
    }
  });

  return (
    <div>
      <DashboardNavbar />
      <Container id="project-container">
        <ProjectHeader projectName={projectName} />
        <ProjectTabs projectName={projectName} activeKey="overview" />
        <div id="project-overview-container" className="mt-4">
          <Container>
            <Row>
              <Col lg={8} md={12} id="project-overview-left">
                {projectOverview && projectOverview.prodDeploymentId ? (
                  <ProjectProdDeploymentCard
                    deployment={projectOverview.prodDeployment || {}}
                  />
                ) : (
                  <ProjectNoDeploymentCard />
                )}
                <Card className="project-deployment-card">
                  <Card.Body>
                    <Card.Title className="d-flex justify-content-between">
                      <div>
                        Latest Deployments
                        <span className="text-muted small fs-6 deployment-recent fw-normal d-block py-2">
                          the recent 10 deployments
                        </span>
                      </div>
                      <div className="deployment-show-all">
                        <ButtonLink to="./deployments" variant="light">
                          Show all
                        </ButtonLink>
                      </div>
                    </Card.Title>
                    <ProjectDeploymentsListGroup
                      deploymentsList={projectOverview?.deploymentsList || []}
                    />
                  </Card.Body>
                </Card>
              </Col>
              <Col
                lg={4}
                md="auto"
                className="d-none d-lg-block d-xl-block d-xxl-block"
                id="project-overview-right"
              >
                <Card className="project-tips mb-3">
                  <Card.Body>
                    <Card.Title>Tips</Card.Title>
                    <p className="text-muted">// TODO</p>
                  </Card.Body>
                </Card>
              </Col>
            </Row>
          </Container>
        </div>
      </Container>
    </div>
  );
}

export default ProjectPage;
