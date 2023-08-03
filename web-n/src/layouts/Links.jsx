// code from https://github.com/react-bootstrap/code-sandbox-examples/blob/master/ts-react-router-v6/src/ButtonLink.tsx

import { Dropdown, Button, Nav, NavDropdown } from "react-bootstrap";
import { useHref, useLinkClickHandler } from "react-router-dom";
import * as React from "react";

const DropdownItemLink = React.forwardRef(
  ({ onClick, replace = false, state, target, to, ...props }, ref) => {
    let href = useHref(to);
    let handleClick = useLinkClickHandler(to, { replace, state, target });
    return (
      <Dropdown.Item
        {...props}
        href={href}
        onClick={(event) => {
          onClick?.(event);
          if (!event.defaultPrevented) {
            handleClick(event);
          }
        }}
        ref={ref}
        target={target}
      />
    );
  }
);

const NavbarLink = React.forwardRef(
  ({ onClick, replace = false, state, target, to, ...props }, ref) => {
    let href = useHref(to);
    let handleClick = useLinkClickHandler(to, { replace, state, target });
    return (
      <Nav.Link
        {...props}
        href={href}
        onClick={(event) => {
          onClick?.(event);
          if (!event.defaultPrevented) {
            handleClick(event);
          }
        }}
        ref={ref}
        target={target}
      />
    );
  }
);

const NavDropdownLink = React.forwardRef(
  ({ onClick, replace = false, state, target, to, ...props }, ref) => {
    let href = useHref(to);
    let handleClick = useLinkClickHandler(to, { replace, state, target });
    return (
      <NavDropdown.Item
        {...props}
        href={href}
        onClick={(event) => {
          onClick?.(event);
          if (!event.defaultPrevented) {
            handleClick(event);
          }
        }}
        ref={ref}
        target={target}
      />
    );
  }
);

const ButtonLink = React.forwardRef(
  ({ onClick, replace = false, state, target, to, ...props }, ref) => {
    let href = useHref(to);
    let handleClick = useLinkClickHandler(to, { replace, state, target });
    return (
      <Button
        {...props}
        href={href}
        onClick={(event) => {
          onClick?.(event);
          if (!event.defaultPrevented) {
            handleClick(event);
          }
        }}
        ref={ref}
        target={target}
      />
    );
  }
);

export { DropdownItemLink, NavbarLink, NavDropdownLink, ButtonLink };
