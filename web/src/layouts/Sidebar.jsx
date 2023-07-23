import { Nav, Image, Dropdown } from "react-bootstrap";
import {
  VscProject,
  VscSymbolMisc,
  VscFeedback,
  VscSettingsGear,
  VscPreview,
  VscGraphLine,
  VscDebugLineByLine,
} from "react-icons/vsc";
import { useLocation } from "react-router-dom";
import { useAuthContext } from "../contexts/Auth";
import { useClerk } from "@clerk/clerk-react";
import { DropdownItemLink, NavbarLink } from "../contexts/Link";

function SidebarLogo() {
  return (
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
  );
}

function SidebarBottonNav({ activeKey }) {
  return (
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
  );
}

function SidebarAccount() {
  const { user } = useAuthContext();
  const { signOut } = useClerk();

  const handleSignOut = (event) => {
    event.preventDefault();
    signOut();
  };

  return (
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
              src={user.avatar_url}
              rounded
              width={30}
              height={30}
              className="mx-2"
            />
            <span className="fw-bold">{user.name}</span>
          </span>
        </Dropdown.Toggle>
        <Dropdown.Menu>
          <DropdownItemLink to="/account">Account</DropdownItemLink>
          <Dropdown.Divider />
          <Dropdown.Item href="/#sign-out" onClick={handleSignOut}>
            Sign Out
          </Dropdown.Item>
        </Dropdown.Menu>
      </Dropdown>
      <div className="footer text-center fs-6 text-body-tertiary">
        @2023 Runtime.land
      </div>
    </div>
  );
}

function DefaultSidebar() {
  function locationNavActiveKey(pathname) {
    // if pathname is /projects/123, return /projects
    // if pathname is /projects, return /projects
    // if pathname is /projects/123/deployments, return /projects
    if (pathname.startsWith("/projects")) {
      return "/projects";
    }
    return pathname;
  }

  let location = useLocation();
  let activeKey = locationNavActiveKey(location.pathname);

  return (
    <div className="d-flex flex-column flex-shrink-0 border-end sidebar-container p-3">
      <SidebarLogo />
      <Nav
        variant="pills"
        defaultActiveKey={activeKey}
        className="flex-column sidebar-nav mb-auto px-3"
      >
        <NavbarLink to="/projects">
          <VscProject className="me-2" />
          Projects
        </NavbarLink>
        <NavbarLink to="/settings">
          <VscSettingsGear className="me-2" />
          Settings
        </NavbarLink>
      </Nav>
      <hr className="divider mx-3" />
      <SidebarBottonNav activeKey={activeKey} />
      <SidebarAccount />
    </div>
  );
}

function ProjectSidebar({ projectName }) {
  function locationNavActiveKey(pathname) {
    // if pathname is /projects/123, return /projects
    // if pathname is /projects, return /projects
    // if pathname is /projects/123/deployments, return /projects
    if (pathname.startsWith("/projects")) {
      return "/projects";
    }
    return pathname;
  }

  let location = useLocation();
  let activeKey = locationNavActiveKey(location.pathname);

  return (
    <div className="d-flex flex-column flex-shrink-0 border-end sidebar-container p-3">
      <SidebarLogo />
      <Nav
        variant="pills"
        defaultActiveKey={activeKey}
        className="flex-column sidebar-nav mb-auto px-3"
      >
        <Nav.Link href={"/projects/" + projectName + "/overview"}>
          <VscPreview className="me-2" />
          Overview
        </Nav.Link>
        <Nav.Link disabled href={"/projects/" + projectName + "/traffict"}>
          <VscGraphLine className="me-2" />
          Traffic
        </Nav.Link>
        <Nav.Link disabled href={"/projects/" + projectName + "/logs"}>
          <VscDebugLineByLine className="me-2" />
          Logs
        </Nav.Link>
        <Nav.Link href={"/projects/" + projectName + "/settings"}>
          <VscSettingsGear className="me-2" />
          Settings
        </Nav.Link>
      </Nav>
      <hr className="divider mx-3" />
      <SidebarBottonNav activeKey={activeKey} />
      <SidebarAccount />
    </div>
  );
}

export { DefaultSidebar, ProjectSidebar };
