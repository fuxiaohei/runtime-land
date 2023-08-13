import { Spinner } from "react-bootstrap";
import { FiAlertTriangle } from "react-icons/fi";

function LoadingLine() {
  return (
    <div className="lh-lg text-secondary">
      <Spinner animation="border" size="sm" />
      <span className="ms-3">Loading...</span>
    </div>
  );
}

function ErrorLine({ error }) {
  return (
    <div className="lh-lh text-danger">
      <FiAlertTriangle size={20} />
      <span className="ms-3">{String(error)}</span>
    </div>
  );
}

function QueryWrapper({ isLoading, isError, error, children }) {
  if (isLoading) {
    return <LoadingLine />;
  }
  if (isError) {
    return <ErrorLine error={error} />;
  }
  return children;
}

export default QueryWrapper;
