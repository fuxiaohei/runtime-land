function LoginLayout(props) {
  return <main className="text-center">{props.children}</main>;
}

function LoginSidebar() {
  return (
    <div className="login-left-side">
      <p>
        <img alt="" src="/public/logo-v2-small.svg" width="64" height="64" />
      </p>
      <h3>Runtime.land</h3>
      <p>Function as a Service</p>
    </div>
  );
}

export { LoginLayout, LoginSidebar };
