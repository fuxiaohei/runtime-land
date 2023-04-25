/**
 * This represents some generic auth provider API, like Firebase.
 */
const mockAuthProvider = {
  isAuthenticated: false,
  signin: (callback) => {
    this.isAuthenticated = true;
    setTimeout(callback, 100); // fake async
  },
  signout: (callback) => {
    this.isAuthenticated = false;
    setTimeout(callback, 100);
  },
};

export { mockAuthProvider };
