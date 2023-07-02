import { Nav, Image, Dropdown } from "react-bootstrap";
import {
  VscProject,
  VscSymbolMisc,
  VscFeedback,
  VscSettingsGear,
} from "react-icons/vsc";
import { useLocation } from "react-router-dom";

function locationNavActiveKey(pathname) {
  // if pathname is /projects/123, return /projects
  // if pathname is /projects, return /projects
  // if pathname is /projects/123/deployments, return /projects
  if (pathname.startsWith("/projects")) {
    return "/projects";
  }
  return pathname;
}

function Sidebar() {
  let location = useLocation();
  let activeKey = locationNavActiveKey(location.pathname);
  return (
    <div className="d-flex flex-column flex-shrink-0 border-end sidebar-container p-3">
      <div className="logo fs-4 fw-bold text-center">
        <img
          alt=""
          src="/public/logo-v2-small.svg"
          width="64"
          height="64"
          className="d-inline-block align-top me-1"
        />{" "}
        <hr className="divider mx-3" />
      </div>
      <Nav
        variant="pills"
        defaultActiveKey={activeKey}
        className="flex-column sidebar-nav mb-auto px-3"
      >
        <Nav.Link href="/projects">
          <VscProject className="me-2" />
          Projects
        </Nav.Link>
        <Nav.Link href="/settings">
          <VscSettingsGear className="me-2" />
          Settings
        </Nav.Link>
      </Nav>
      <hr className="divider mx-3" />
      <Nav
        variant="pills"
        defaultActiveKey={activeKey}
        className="flex-column sidebar-nav px-3"
      >
        <Nav.Link href="https://runtime.land" target="_blank">
          <VscSymbolMisc className="me-2" />
          Docs
        </Nav.Link>
        <Nav.Link eventKey="link-1">
          <VscFeedback className="me-2" />
          Feedback
        </Nav.Link>
      </Nav>
      <div className="account d-flex flex-column">
        <hr className="divider mx-3" />
        <Dropdown drop="end" align="end">
          <Dropdown.Toggle
            as="div"
            role="button"
            className="account-avatar mb-5 px-3"
          >
            <span className="me-2">
              <Image
                src="https://avatars.githubusercontent.com/u/2142787?v=4"
                rounded
                width={30}
                height={30}
                className="mx-2"
              />
              <span className="fw-bold">FuXiaohei</span>
            </span>
          </Dropdown.Toggle>
          <Dropdown.Menu>
            <Dropdown.Item href="/account">Account</Dropdown.Item>
            <Dropdown.Divider />
            <Dropdown.Item href="/sign-out">Sign Out</Dropdown.Item>
          </Dropdown.Menu>
        </Dropdown>
        <div className="footer text-center fs-6 text-body-tertiary">
          @2023 Runtime.land
        </div>
      </div>
    </div>
  );
}

export default Sidebar;
