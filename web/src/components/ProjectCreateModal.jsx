import { useState } from "react";
import { Alert, Button, Form, InputGroup, Modal } from "react-bootstrap";
import { BiRefresh } from "react-icons/bi";
import {
  NumberDictionary,
  adjectives,
  colors,
  uniqueNamesGenerator,
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

function ProjectCreateModal({ show, handleClose, handleCreate, alert }) {
  const [validated, setValidated] = useState(false);
  const [autoName, setAutoName] = useState(generateName());
  const [language, setLanguage] = useState("rust");

  const handleSubmit = async (event) => {
    const form = event.currentTarget;
    if (form.checkValidity() === false) {
      event.preventDefault();
      event.stopPropagation();
    }

    setValidated(true);
    event.preventDefault();
    await handleCreate({ name: autoName, language: language });
    refreshName();
  };

  const refreshName = () => {
    setAutoName(generateName());
  };

  return (
    <Modal show={show} onHide={handleClose}>
      <Modal.Header closeButton>
        <Modal.Title>Create new project</Modal.Title>
      </Modal.Header>
      <Form noValidate validated={validated} onSubmit={handleSubmit}>
        <Modal.Body>
          <Form.Group className="mb-3" controlId="project-name-input">
            <Form.Label>Project name</Form.Label>
            <InputGroup className="mb-3">
              <Form.Control
                placeholder="project name"
                required
                value={autoName}
                onChange={(e) => setAutoName(e.target.value)}
              />
              <Button
                className="rounded-end"
                variant="outline-secondary"
                onClick={refreshName}
              >
                <BiRefresh />
              </Button>
              <Form.Control.Feedback type="invalid">
                Please provide a valid project name.
              </Form.Control.Feedback>
            </InputGroup>
          </Form.Group>
          <Form.Group className="mb-3" controlId="project-name-language">
            <Form.Label>Project language</Form.Label>
            <Form.Select
              onChange={(e) => {
                setLanguage(e.target.value);
              }}
            >
              <option value="rust">Rust</option>
              <option value="java">Java</option>
              <option value="golang">Go</option>
            </Form.Select>
          </Form.Group>
          {alert ? <Alert variant="danger">{alert}</Alert> : null}
        </Modal.Body>
        <Modal.Footer>
          <Button variant="secondary" onClick={handleClose}>
            Cancel
          </Button>
          <Button variant="primary" type="submit">
            Create
          </Button>
        </Modal.Footer>
      </Form>
    </Modal>
  );
}

export default ProjectCreateModal;
