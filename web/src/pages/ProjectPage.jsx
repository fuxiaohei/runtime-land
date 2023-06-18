import { Container, Row, Col, Card } from "react-bootstrap";
import DashboardNavbar from "../components/DashboardNavbar";
import { ButtonLink } from "../components/ButtonLink";
import ProjectHeader from "../components/ProjectHeader";
import ProjectTabs from "../components/ProjectTabs";
import { useParams } from "react-router-dom";
import { getProjectOverview, publishDeployment } from "../api/project";
import React, { useEffect } from "react";
import ProjectNoDeploymentCard from "../components/ProjectNoDeploymentCard";
import ProjectProdDeploymentCard from "../components/ProjectProdDeploymentCard";
import ProjectDeploymentsListGroup from "../components/ProjectDeploymentsListGroup";
import DeployToProductionModal from "../components/DeployToProductionModal";

function ProjectPage() {
  const { projectName } = useParams();
  const [projectOverview, setProjectOverview] = React.useState(null);
  const [showDeployToProduction, setShowDeployToProduction] =
    React.useState(false);
  const [currentDeployToProduction, setCurrentDeployToProduction] =
    React.useState(null);
  const [loadingDeployToProduction, setLoadingDeployToProduction] =
    React.useState(false);

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

  const handleDeployToProduction = async (deployment) => {
    setCurrentDeployToProduction(deployment);
    setShowDeployToProduction(true);
  };

  const handleDeployToProductionCancel = async () => {
    setShowDeployToProduction(false);
  };

  const handleDeployToProductionConfirm = async (event) => {
    setLoadingDeployToProduction(true);
    let response = await publishDeployment(
      currentDeployToProduction.id,
      currentDeployToProduction.uuid
    );
    if (response.error) {
      // FIXME: show error message
      return;
    }
    await fetchProjectOverview(); // refresh project overview
    setLoadingDeployToProduction(false);
    setShowDeployToProduction(false);
  };

  return (
    <div>
      <DashboardNavbar />
      <Container id="project-container">
        <ProjectHeader
          projectName={projectName}
          project={projectOverview || {}}
        />
        <ProjectTabs projectName={projectName} activeKey="overview" />
        <div id="project-overview-container" className="pt-4 pb-5">
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
                      onDeployToProduction={handleDeployToProduction}
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
      <DeployToProductionModal
        show={showDeployToProduction}
        loading={loadingDeployToProduction.toString()}
        current={currentDeployToProduction || {}}
        prev={projectOverview?.prodDeployment || {}}
        produrl={projectOverview?.prodUrl || ""}
        onCancel={handleDeployToProductionCancel}
        onSubmit={handleDeployToProductionConfirm}
      />
    </div>
  );
}

export default ProjectPage;
