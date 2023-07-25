import { Breadcrumb, Container, Nav } from "react-bootstrap";
import { DefaultSidebar, ProjectSidebar } from "./Sidebar";
import { VscBell } from "react-icons/vsc";
import { Link, useLocation } from "react-router-dom";
import { Helmet } from "react-helmet-async";
import { ToastProvider } from "../contexts/Toast";

function MainBreadcrumb() {
  let location = useLocation();

  const renderBreadcrumb = () => {
    console.log("location", location, location.pathname.startsWith("/account"));
    if (location.pathname.startsWith("/account")) {
      return <Breadcrumb.Item active>Account</Breadcrumb.Item>;
    }
    if (location.pathname.startsWith("/projects")) {
      return <Breadcrumb.Item active>Projects</Breadcrumb.Item>;
    }
    return null;
  };

  return (
    <div className="main-breadcrumb d-flex flex-row justify-content-between p-3 border-bottom">
      <Breadcrumb>
        <Breadcrumb.Item linkAs="span">
          <Link to="/projects">Home</Link>
        </Breadcrumb.Item>
        {renderBreadcrumb()}
      </Breadcrumb>
      <Nav>
        <Nav.Link>
          <VscBell size={18} />
        </Nav.Link>
      </Nav>
    </div>
  );
}

function DefaultLayout({ title, children }) {
  return (
    <ToastProvider>
      <main className="d-flex flex-nowrap">
        <Helmet>
          <title>{title}</title>
        </Helmet>
        <DefaultSidebar />
        <Container fluid className="main-container">
          <MainBreadcrumb />
          <div className="main-section p-3">{children}</div>
        </Container>
      </main>
    </ToastProvider>
  );
}

function ProjectLayout({ title, children, projectName }) {
  console.log("projectName", projectName);
  return (
    <main className="d-flex flex-nowrap">
      <Helmet>
        <title>{title}</title>
      </Helmet>
      <ProjectSidebar projectName={projectName} />
      <Container fluid className="main-container">
        <MainBreadcrumb />
        <div className="main-section p-3">{children}</div>
      </Container>
    </main>
  );
}

export { DefaultLayout, ProjectLayout };
