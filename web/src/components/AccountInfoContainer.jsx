import { Col, Form } from "react-bootstrap";
import { useAuthContext } from "../contexts/Auth";

function AccountInfoContainer() {
  const { user } = useAuthContext();
  return (
    <Col md={4}>
      <h4 className="mt-3 account-info-title">Account Infomation</h4>
      <hr />
      <Form>
        <Form.Group className="mb-3">
          <Form.Label>Name</Form.Label>
          <Form.Control type="text" disabled defaultValue={user.name} />
        </Form.Group>
        <Form.Group className="mb-3">
          <Form.Label>Email</Form.Label>
          <Form.Control type="email" disabled defaultValue={user.email} />
        </Form.Group>
      </Form>
    </Col>
  );
}

export default AccountInfoContainer;
