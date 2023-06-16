import { Container } from "react-bootstrap";
import DashboardNavbar from "../components/DashboardNavbar";
import ProjectHeader from "../components/ProjectHeader";
import ProjectTabs from "../components/ProjectTabs";
import { useParams } from "react-router-dom";

function ProjectSettingsPage() {
  let { projectName } = useParams();
  return (
    <div>
      <DashboardNavbar />
      <Container id="project-container">
        <ProjectHeader projectName={projectName} />
        <ProjectTabs projectName={projectName} activeKey="settings" />
        <div id="project-overview-container" className="pt-4 pb-5">
          <Container>TODO:(settings)</Container>
        </div>
      </Container>
    </div>
  );
}

export default ProjectSettingsPage;
