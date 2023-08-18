import { Alert, Button, Form } from "react-bootstrap";

function LocalStorageForm({ data }) {
  return (
    <div className="storage-local">
      <Form.Group className="mb-3">
        <Form.Label>Local Storage Path</Form.Label>
        <Form.Control type="text" disabled defaultValue={data.path} />
        <Form.Text className="text-muted">
          Enter the path where your project's files will be stored
        </Form.Text>
      </Form.Group>
      <div className="text-start">
        <Button
          className="d-inline-block"
          variant="primary"
          type="submit"
          disabled
        >
          Save
        </Button>
        <Alert className="mt-3 p-2 ms-4 d-inline-block" variant="danger">
          Local storage is not supported to change at the moment.
        </Alert>
      </div>
    </div>
  );
}

function S3StorageForm({ data }) {
  return (
    <div className="storage-s3">
      <Form.Group className="mb-3">
        <Form.Label>S3 Endpoint</Form.Label>
        <Form.Control
          name="endpoint"
          type="text"
          defaultValue={data.endpoint}
          required
        />
        <Form.Text className="text-muted">
          Enter the endpoint of your S3 storage provider
        </Form.Text>
      </Form.Group>
      <Form.Group className="mb-3">
        <Form.Label>S3 Bucket</Form.Label>
        <Form.Control
          name="bucket"
          type="text"
          defaultValue={data.bucket}
          required
        />
        <Form.Text className="text-muted">
          Enter the bucket of your S3 storage provider
        </Form.Text>
      </Form.Group>
      <Form.Group className="mb-3">
        <Form.Label>S3 Region</Form.Label>
        <Form.Control
          name="region"
          type="text"
          defaultValue={data.region}
          required
        />
        <Form.Text className="text-muted">
          Enter the region of your S3 storage provider
        </Form.Text>
      </Form.Group>
      <Form.Group className="mb-3">
        <Form.Label>Access Key ID</Form.Label>
        <Form.Control
          name="access_key_id"
          type="text"
          defaultValue={data.access_key_id}
          required
        />
        <Form.Text className="text-muted">
          Enter the access key ID of your S3 storage provider
        </Form.Text>
      </Form.Group>
      <Form.Group className="mb-3">
        <Form.Label>Secret Access Key</Form.Label>
        <Form.Control
          name="secret_access_key"
          type="text"
          defaultValue={data.secret_access_key}
          required
        />
        <Form.Text className="text-muted">
          Enter the secret access key of your S3 storage provider
        </Form.Text>
      </Form.Group>
      <Form.Group className="mb-3">
        <Form.Label>Root path</Form.Label>
        <Form.Control
          name="root_path"
          type="text"
          defaultValue={data.root_path}
          required
        />
        <Form.Text className="text-muted">
          Enter the root path to store your project's files
        </Form.Text>
      </Form.Group>
      <div className="text-start">
        <Button className="d-inline-block" variant="primary" type="submit">
          Save
        </Button>
      </div>
    </div>
  );
}

function AdminStorageForm({ data, onSubmit, isSuccess }) {
  const subForm =
    data?.storage_type === "local" ? (
      <LocalStorageForm data={data.local} />
    ) : (
      <S3StorageForm data={data.s3} />
    );

  const handleSubmit = (e) => {
    e.preventDefault();
    const formData = new FormData(e.target);
    const values = Object.fromEntries(formData.entries());
    onSubmit({ typename: data.storage_type, storage: values });
  };

  return (
    <Form
      id="storage-form"
      className="border-top mt-4 pt-4"
      onSubmit={handleSubmit}
    >
      {isSuccess ? (
        <Alert className="mb-3" variant="success" dismissible>
          Settings updated successfully
        </Alert>
      ) : null}
      <Form.Group className="mb-3">
        <Form.Label>Storage Provider</Form.Label>
        <Form.Control
          name="storage_type"
          type="text"
          readOnly
          defaultValue={data?.storage_type}
          disabled
        />
        <Form.Text className="text-muted">
          Select the storage provider for storing your project's files
        </Form.Text>
      </Form.Group>
      {subForm}
    </Form>
  );
}

export default AdminStorageForm;
