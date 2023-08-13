import { Nav } from "react-bootstrap";
import { NavbarLink } from "../layouts/Links";

function AdminNavHeader({ activeKey }) {
  return (
    <Nav className="my-3" variant="underline" defaultActiveKey={activeKey}>
      <Nav.Item>
        <NavbarLink className="me-3" eventKey="stats" to="/admin/stats">
          Stats
        </NavbarLink>
      </Nav.Item>
      <Nav.Item>
        <NavbarLink className="me-3" eventKey="regions" to="/admin/regions">
          Regions
        </NavbarLink>
      </Nav.Item>
      <Nav.Item>
        <NavbarLink className="me-3" eventKey="domains" to="/admin/domains">
          Domains
        </NavbarLink>
      </Nav.Item>
      <Nav.Item>
        <NavbarLink className="me-3" eventKey="storage" to="/admin/storage">
          Storage
        </NavbarLink>
      </Nav.Item>
    </Nav>
  );
}

export default AdminNavHeader;
