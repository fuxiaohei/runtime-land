import { Container, Nav, Breadcrumb } from "react-bootstrap";
import { DefaultSidebar } from "./Sidebar";
import { Helmet } from "react-helmet-async";
import { ToastProvider } from "../contexts/Toast";
import { Link, useLocation } from "react-router-dom";
import { VscBell } from "react-icons/vsc";

function AdminBreadcrumb() {
  let location = useLocation();

  const renderBreadcrumb = () => {
    if (location.pathname.startsWith("/admin")) {
      return <Breadcrumb.Item active>Admin</Breadcrumb.Item>;
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

function AdminLayout({ title, children }) {
  return (
    <ToastProvider>
      <main className="d-flex flex-nowrap">
        <Helmet>
          <title>{title}</title>
        </Helmet>
        <DefaultSidebar />
        <Container fluid className="main-container">
          <AdminBreadcrumb />
          <div className="main-section p-3">{children}</div>
        </Container>
      </main>
    </ToastProvider>
  );
}

export default AdminLayout;
