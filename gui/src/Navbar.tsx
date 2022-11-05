import React, { Component, ReactNode } from "react";
import "./Navbar.scss";

type NavbarTabProps = {
    active: boolean;
    name: string;
    onClick: (tab: string) => void;
};

class NavbarTab extends Component<NavbarTabProps, {}> {
    onClick = (): void => {
        this.props.onClick(this.props.name);
    };

    render(): ReactNode {
        let active: string = this.props.active ? "active" : "";
        let src: string = this.props.name.toLowerCase() + ".svg";
        return (
            <li className={active} onClick={this.onClick}>
                <img src={src} alt={this.props.name} />
                <span>{this.props.name}</span>
            </li>
        );
    }
}

type NavbarProps = {
    active_tab: string;
    onTab: (tab: string) => void;
    items: string[];
};

class Navbar extends Component<NavbarProps, {}> {
    render(): ReactNode {
        let columns = {
            gridTemplateColumns:
                "repeat(" + this.props.items.length + ", auto)",
        };
        let content = this.props.items.map((x) => {
            let active = this.props.active_tab === x;
            return (
                <NavbarTab
                    key={x}
                    name={x}
                    onClick={this.props.onTab}
                    active={active}
                />
            );
        });
        return (
            <ul className="navbar" style={columns}>
                {content}
            </ul>
        );
    }
}

export default Navbar;
