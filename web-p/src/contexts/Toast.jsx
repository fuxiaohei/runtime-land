import React, { useState } from "react";
import { Toast, ToastContainer } from "react-bootstrap";

const ToastContext = React.createContext(null);

function useToastContext() {
  return React.useContext(ToastContext);
}

function ToastProvider({ children }) {
  const [show, setShow] = useState(false);
  const [message, setMessage] = useState({});
  const value = {
    toastInfo: (content) => {
      setMessage({ title: "Info", content, variant: "secondary" });
      setShow(true);
    },
    toastSuccess: (content) => {
      setMessage({ title: "Success", content, variant: "success" });
      setShow(true);
    },
    toastError: (content) => {
      setMessage({ title: "Error", content, variant: "danger" });
      setShow(true);
    },
  };
  return (
    <ToastContext.Provider value={value}>
      {children}
      <ToastContainer className="p-3" position="top-end" style={{ zIndex: 1 }}>
        <Toast
          bg={message.variant}
          onClose={() => setShow(false)}
          autohide={true}
          delay={3000}
          show={show}
        >
          <Toast.Header closeButton={false}>
            <strong className="me-auto">{message.title}</strong>
          </Toast.Header>
          <Toast.Body>{message.content}</Toast.Body>
        </Toast>
      </ToastContainer>
    </ToastContext.Provider>
  );
}

export { ToastProvider, useToastContext };
