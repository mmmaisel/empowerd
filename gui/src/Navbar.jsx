import React, {Component} from 'react';
import './Navbar.scss';

class NavbarTab extends Component
{
    onClick = () =>
    {
        this.props.onClick(this.props.name);
    }

    render()
    {
        let active = this.props.active ? "active" : "";
        let src = this.props.name.toLowerCase() + ".svg";
        return (
            <li className={ active } onClick={ this.onClick }>
                <img src={ src } alt={ this.props.name } />
                <span>{ this.props.name }</span>
            </li>
        );
    }
}

class Navbar extends Component
{
    render()
    {
        let columns =
        {
            gridTemplateColumns:
                "repeat(" + this.props.items.length + ", auto)"
        };
        let content = this.props.items.map((x) =>
        {
            let active = this.props.active_tab === x;
            return <NavbarTab key={ x } name={ x }
                onClick={ this.props.onTab } active={ active } />;
        });
        return (
            <ul className="navbar" style={ columns }>
                { content }
            </ul>
        );
    }
}

export default Navbar;
