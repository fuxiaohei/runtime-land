import { Container } from "react-bootstrap";
import DashboardNavbar from "../components/DashboardNavbar";
import ProjectHeader from "../components/ProjectHeader";
import ProjectTabs from "../components/ProjectTabs";
import { useParams } from "react-router-dom";

function ProjectDeploymentPage() {
  let { projectName } = useParams();
  return (
    <div>
      <DashboardNavbar />
      <Container id="project-container">
        <ProjectHeader projectName={projectName} />
        <ProjectTabs projectName={projectName} activeKey="deployments" />
        <div id="project-overview-container" className="mt-4">
          <Container>deployment</Container>
        </div>
      </Container>
    </div>
  );
}

export default ProjectDeploymentPage;
