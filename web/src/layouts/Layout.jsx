import { Breadcrumb, Container, Nav } from "react-bootstrap";
import { DefaultSidebar, ProjectSidebar } from "./Sidebar";
import { VscBell } from "react-icons/vsc";

function MainBreadcrumb() {
  return (
    <div className="main-breadcrumb d-flex flex-row justify-content-between p-3 border-bottom">
      <Breadcrumb>
        <Breadcrumb.Item href="/projects">Home</Breadcrumb.Item>
        <Breadcrumb.Item active>polite-pike-746</Breadcrumb.Item>
      </Breadcrumb>
      <Nav>
        <Nav.Link>
          <VscBell size={18} />
        </Nav.Link>
      </Nav>
    </div>
  );
}

function DefaultLayout({ children }) {
  return (
    <main className="d-flex flex-nowrap">
      <DefaultSidebar />
      <Container fluid className="main-container">
        <MainBreadcrumb />
        <div className="main-section p-3">{children}</div>
      </Container>
    </main>
  );
}

function ProjectLayout({ children, projectName }) {
  console.log("projectName", projectName)
  return (
    <main className="d-flex flex-nowrap">
      <ProjectSidebar projectName={projectName} />
      <Container fluid className="main-container">
        <MainBreadcrumb />
        <div className="main-section p-3">{children}</div>
      </Container>
    </main>
  );
}

export { DefaultLayout, ProjectLayout };
