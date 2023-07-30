import { useState } from "react";
import { Form, Button, Modal, Alert, InputGroup } from "react-bootstrap";
import { BiRefresh } from "react-icons/bi";
import {
  uniqueNamesGenerator,
  NumberDictionary,
  adjectives,
  colors,
} from "unique-names-generator";

const generateName = () => {
  const numberDictionary = NumberDictionary.generate({ min: 100, max: 999 });
  const shortName = uniqueNamesGenerator({
    dictionaries: [adjectives, colors, numberDictionary],
    length: 3,
    separator: "-",
  });
  return shortName;
};

function ProjectCreateModal(props) {
  const [validated, setValidated] = useState(false);
  const [projectName, setProjectName] = useState(generateName());
  const [projectLanguage, setProjectLanguage] = useState("rust");

  const handleSubmit = async (event) => {
    const form = event.currentTarget;
    const validated = form.checkValidity();
    if (validated === false) {
      event.preventDefault();
      setValidated(true);
      return;
    }
    setValidated(true);
    event.preventDefault();

    await props.onCreate({ name: projectName, language: projectLanguage });
    refreshName();
  };

  const refreshName = () => {
    setProjectName(generateName());
  };

  return (
    <Modal show={props.show}>
      <Modal.Header>
        <Modal.Title>Create new project</Modal.Title>
      </Modal.Header>
      <Form noValidate validated={validated} onSubmit={handleSubmit}>
        <Modal.Body>
          <Form.Group className="mb-3">
            <div className="mb-3">
              <Form.Text className="text-muted">
                Enter your project name.
              </Form.Text>
            </div>
            <InputGroup>
              <Form.Control
                name="tokenvalue"
                required
                type="text"
                placeholder="What's the project name"
                value={projectName}
                onChange={(e) => setProjectName(e.target.value)}
              />
              <Button size="sm" variant="outline-success" onClick={refreshName}>
                <BiRefresh size={20} />
              </Button>
            </InputGroup>
            <Form.Control.Feedback type="invalid">
              Please enter a valid project name.
            </Form.Control.Feedback>
          </Form.Group>
          <Form.Group className="mb-3">
            <div className="mb-3">
              <Form.Text className="text-muted">
                Select the language of your project.
              </Form.Text>
            </div>
            <Form.Select
              onChange={(e) => {
                setProjectLanguage(e.target.value);
              }}
            >
              <option value="rust">Rust</option>
            </Form.Select>
          </Form.Group>
          {props.alert ? (
            <Alert dismissible variant="danger">
              {props.alert}
            </Alert>
          ) : null}
        </Modal.Body>
        <Modal.Footer>
          <Button variant="secondary" size="sm" onClick={props.onClose}>
            Cancel
          </Button>
          <Button type="submit" variant="primary" size="sm">
            Create
          </Button>
        </Modal.Footer>
      </Form>
    </Modal>
  );
}

export default ProjectCreateModal;
