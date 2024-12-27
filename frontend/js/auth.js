document.addEventListener("DOMContentLoaded", () => {
    const loginForm = document.getElementById("login-form");
    const registerForm = document.getElementById("register-form");
    const toggleFormButton = document.getElementById("toggle-form-button");
    const loginWithEmailButton = document.getElementById("login-with-email");
    const hiddenLoginTypeField = document.getElementById('login-type');

    toggleFormButton.addEventListener("click", () => {
        if (loginForm.style.display === "none") {
            loginForm.style.display = "flex";
            loginWithEmailButton.style.display = "block";
            registerForm.style.display = "none";
            toggleFormButton.innerText = "Register";
        } else {
            loginForm.style.display = "none";
            loginWithEmailButton.style.display = "none";
            registerForm.style.display = "flex";
            toggleFormButton.innerText = "Back to Login";
        }
    });

    loginWithEmailButton.addEventListener("click", () => {
        const usernameField = document.getElementById("username");
        const usernameLabel = document.querySelector("label[for='username']");

        if (usernameField.placeholder === "Enter your username") {
            usernameLabel.textContent = "Email";
            usernameField.placeholder = "Enter your email";
            usernameField.type = "email";
            hiddenLoginTypeField.value = "Email";
            loginWithEmailButton.innerText = "Login with Username";
        } else {
            usernameLabel.textContent = "Username";
            usernameField.placeholder = "Enter your username";
            usernameField.type = "text";
            hiddenLoginTypeField.value = "Username";
            loginWithEmailButton.innerText = "Login with Email";
        }
    });

    attachCustomFormHandler(loginForm);
    attachCustomFormHandler(registerForm);
});

function attachCustomFormHandler(form) {
    form.addEventListener('submit', async (event) => {
        event.preventDefault();

        const formData = new FormData(form);

        try {
            const response = await fetch(form.action, {
                method: form.method,
                body: formData
            });

            if (response.ok) {
                const data = await response.json();
                if ("ErrorMessage" in data) {
                    showToast(data.ErrorMessage);
                }
                else if ("RedirectURL" in data) {
                    window.location.href = data.RedirectURL;
                }
                else {
                    console.log("Invalid Response!");
                }
            } else {
                console.error('Form submission failed:', response.status, response.statusText);
            }
        } catch (error) {
            console.error('Error submitting form:', error);
        }
    });
}

function showToast(message) {
    const container = document.getElementById('toast-container');

    const toast = document.createElement('div');
    toast.className = 'toast';
    toast.textContent = message;

    container.appendChild(toast);

    setTimeout(() => {
        toast.remove();
    }, 3000);
}